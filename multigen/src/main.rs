use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Write};

struct DefaultMapObjectInfo {
    spawn_state: String, // Spawn state name
    radius: String,
    height: String,
}

struct State {
    name: String,       // State name
    sprite: String,     // Sprite name
    frame: i16,         // Frame, A=0, B=1, ...
    full_bright: bool,  // Should the sprite be rendered with full brightness?
    tics: i16,          // Tick count
    action: String,     // Function name to call
    next_state: String, // Next state
}

fn write_prologue(output: &mut File) {
    output
        .write_all(b"// Automatically generated, do not edit.\n")
        .unwrap();
}

fn write_sprites(output: &mut File, sprites: &Vec<String>) {
    output
        .write_all(
            br#"

#[repr(i16)]
#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, Clone)]
pub enum Sprite {
"#,
        )
        .unwrap();

    for sprite in sprites {
        output
            .write_all(format!("    {},\n", sprite.to_uppercase()).as_bytes())
            .unwrap();
    }

    output.write_all(b"}\n").unwrap();
}

fn write_states(output: &mut File, states: &Vec<State>) {
    output
        .write_all(
            br#"
#[repr(i16)]
#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum StateId {
"#,
        )
        .unwrap();

    for state in states {
        output
            .write_all(format!("    {},\n", state.name.to_uppercase()).as_bytes())
            .unwrap();
    }

    output.write_all(b"}\n").unwrap();

    output
        .write_all(
            format!(
                r#"
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct State {{
    pub id: StateId,           // State id
    pub sprite: Sprite,
    pub frame: i16,            // Frame, A=0, B=1, ...
    pub full_bright: bool,     // Should the sprite be rendered with full brightness?
    pub tics: i16,             // Tic count
    pub action: &'static str,  // Function name to call
    pub next_state: StateId,   // Next state
}}

#[allow(dead_code)]
pub const STATES: [State; {}] = [
"#,
                states.len()
            )
            .as_bytes(),
        )
        .unwrap();

    for state in states {
        output .write_all( format!(
            "    State{{id: StateId::{}, sprite: Sprite::{}, frame: {}, full_bright: {}, tics: {}, action: {:?}, next_state: StateId::{}}},\n",
            state.name.to_uppercase(), state.sprite, state.frame, state.full_bright, state.tics, state.action, state.next_state.to_uppercase(),
        ).as_bytes()).unwrap();
    }

    output.write_all(b"];\n").unwrap();
}

fn parse_float(string: &str) -> i16 {
    string
        .strip_suffix("*FRACUNIT") // Always present
        .unwrap()
        .parse::<i16>()
        .unwrap()
}

fn parse_default_mobj_info(properties: &HashMap<String, String>) -> DefaultMapObjectInfo {
    DefaultMapObjectInfo {
        spawn_state: properties.get("spawnstate").unwrap().clone(),
        radius: properties.get("radius").unwrap().clone(),
        height: properties.get("height").unwrap().clone(),
    }
}

fn write_mobj_info(
    output: &mut File,
    objects_map: &HashMap<String, HashMap<String, String>>,
    objects_list: &Vec<String>,
) {
    output
        .write_all(
            format!(
                r#"
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MapObjectInfo {{
    pub id: i16,                   // Number to spawn this object
    pub spawn_state: StateId,      // Spawn state id
    pub radius: i16,               // Radius
    pub height: i16,               // Height
}}

#[allow(dead_code)]
pub const MAP_OBJECT_INFOS: [MapObjectInfo; {}] = [
"#,
                objects_list.len()
            )
            .as_bytes(),
        )
        .unwrap();

    let default = parse_default_mobj_info(objects_map.get("DEFAULT").unwrap());

    for object_id in objects_list {
        let properties = objects_map.get(object_id).unwrap();

        let id = properties
            .get("doomednum")
            .unwrap_or(&"-1".to_string())
            .parse::<i16>()
            .unwrap();
        let spawn_state = properties
            .get("spawnstate")
            .unwrap_or(&default.spawn_state)
            .clone();
        let radius = parse_float(properties.get("radius").unwrap_or(&default.radius));
        let height = parse_float(properties.get("height").unwrap_or(&default.height));

        output
            .write_all(
                format!(
                    r#"    MapObjectInfo{{
        id: {},
        spawn_state: StateId::{},
        radius: {},
        height: {},
    }},
"#,
                    id,
                    spawn_state.to_uppercase(),
                    radius,
                    height
                )
                .as_bytes(),
            )
            .unwrap();
    }

    output.write_all(b"];\n").unwrap();
}

fn main() {
    let mut unique_name_counter = 1;
    let mut objects_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut objects_list: Vec<String> = Vec::new();

    let mut sprite_counter = 0;
    let mut sprites_map: HashMap<String, i16> = HashMap::new();
    let mut sprites_list: Vec<String> = Vec::new();

    let mut states: Vec<State> = Vec::new();

    let lines = io::BufReader::new(File::open("multigen.txt").unwrap()).lines();

    let lines = lines
        .flatten()
        .map(|x| x.trim().to_string())
        .map(|mut x| {
            x.replace_range(x.find(";").unwrap_or(x.len()).., "");
            x
        })
        .filter(|x| !x.starts_with(";"))
        .filter(|x| !x.is_empty());

    let mut cur_object_name: String = "".to_string();

    for line in lines {
        if line.starts_with("$") {
            let components: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

            let mut name = components[1].to_string();

            // if MOBJNAME is +, a new unique name will be generated
            if name == "+" {
                name = format!("MT_AUTO_{:03}", unique_name_counter);
                unique_name_counter += 1;
            }
            cur_object_name = name.clone();

            let mut properties = HashMap::new();

            // $ STRING ...
            if components.len() > 2 {
                for pair in components.chunks(2).skip(1) {
                    properties.insert(pair[0].clone(), pair[1].clone());
                }
            }

            objects_map.insert(name.clone(), properties);
            objects_list.push(name);
        } else if line.starts_with("S_") {
            let components: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
            let state_name = components[0].clone();
            let sprite_name = components[1].clone();
            let frame_str = components[2].clone();
            let tics = components[3].trim_end_matches('*').parse::<i16>().unwrap();
            let action = components[4].clone();
            let next_state = components[5].clone();

            if !sprites_map.contains_key(&sprite_name) {
                sprites_map.insert(sprite_name.clone(), sprite_counter);
                sprites_list.push(sprite_name.clone());
                sprite_counter += 1;
            }

            let frame = (frame_str.as_bytes()[0] - b'A') as i16;
            let full_bright = frame_str.contains("*");

            let state = State {
                name: state_name,
                sprite: sprite_name,
                frame,
                full_bright,
                tics,
                action,
                next_state,
            };
            states.push(state);
        } else {
            let properties = objects_map.get_mut(&cur_object_name).unwrap();
            let components: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

            for pair in components.chunks(2) {
                properties.insert(pair[0].clone(), pair[1].clone());
            }
        }
    }

    let mut output = File::create("../src/info.rs").unwrap();
    write_prologue(&mut output);
    write_sprites(&mut output, &sprites_list);
    write_states(&mut output, &states);
    write_mobj_info(&mut output, &objects_map, &objects_list);
}
