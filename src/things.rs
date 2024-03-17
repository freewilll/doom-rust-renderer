use crate::wad::{MapLumpName, WadFile};
use std::rc::Rc;

#[derive(Debug)]
pub enum ThingTypes {
    Player1Start = 1,
}

#[derive(Debug)]
pub struct Thing {
    pub x: f32,
    pub y: f32,
    pub angle: f32, // In radians. 0=East, pi/2=North, pi=West, 3pi/2=South
    pub thing_type: i16,
    pub flags: i16,
}

pub fn load_things(wad_file: &WadFile, map_name: &str) -> Vec<Rc<Thing>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Things);
    let count = dir_entry.size as usize / 10; // A thing is 10 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 10;

        let thing = Thing {
            x: wad_file.read_f32_from_i16(offset),
            y: wad_file.read_f32_from_i16(offset + 2),
            angle: (wad_file.read_f32_from_i16(offset + 4)).to_radians(),
            thing_type: wad_file.read_i16(offset + 6),
            flags: wad_file.read_i16(offset + 8),
        };
        results.push(Rc::new(thing));
    }

    results
}

pub fn get_thing_by_type(things: &Vec<Rc<Thing>>, thing_type: ThingTypes) -> Rc<Thing> {
    let i16_thing_type = thing_type as i16;
    for thing in things {
        if thing.thing_type == i16_thing_type {
            return Rc::clone(thing);
        }
    }

    panic!("Could not find thing of type {}", i16_thing_type);
}
