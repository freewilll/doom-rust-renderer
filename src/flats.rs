use sdl2::rect::Rect;
use std::collections::HashMap;
use std::rc::Rc;
use std::{fmt, str};

use crate::game::Game;
use crate::wad::WadFile;

pub const FLAT_SIZE: i16 = 64;

// Lazy loaded hashmap of flats
pub struct Flats {
    map: HashMap<String, Rc<Flat>>, // The reference counted flats
    wad_file: Rc<WadFile>,          // Needed to be able to lazy load the flats
    animated_flats: HashMap<String, Vec<String>>, // A map of texture name to a list of textures
}

// A 64x64 pixel flat
pub struct Flat {
    pub name: String,
    pub pixels: Vec<Vec<u8>>, // Grid of colormap indexes
}

impl Flats {
    pub fn new(wad_file: &Rc<WadFile>) -> Flats {
        // Lists of animated flats
        // https://doomwiki.org/wiki/Animated_flat
        // Define in doom p_spec.c

        let animated_flats_lists: Vec<Vec<String>> = vec![
            vec!["NUKAGE1".into(), "NUKAGE2".into(), "NUKAGE3".into()],
            vec![
                "FWATER1".into(),
                "FWATER2".into(),
                "FWATER3".into(),
                "FWATER4".into(),
            ],
            vec![
                "SWATER1".into(),
                "SWATER2".into(),
                "SWATER3".into(),
                "SWATER4".into(),
            ],
            vec![
                "LAVA1".into(),
                "LAVA2".into(),
                "LAVA3".into(),
                "LAVA4".into(),
            ],
            vec!["BLOOD1".into(), "BLOOD2".into(), "BLOOD3".into()],
            vec![
                "RROCK05".into(),
                "RROCK06".into(),
                "RROCK07".into(),
                "RROCK08".into(),
            ],
            vec![
                "SLIME01".into(),
                "SLIME02".into(),
                "SLIME03".into(),
                "SLIME04".into(),
            ],
            vec![
                "SLIME05".into(),
                "SLIME06".into(),
                "SLIME07".into(),
                "SLIME08".into(),
            ],
            vec![
                "SLIME09".into(),
                "SLIME10".into(),
                "SLIME11".into(),
                "SLIME12".into(),
            ],
        ];

        // Make a map from texture name to list of belonging textures in the animation
        let mut animated_flats = HashMap::new();
        for list in &animated_flats_lists {
            for texture_name in list {
                animated_flats.insert(texture_name.to_string(), list.clone());
            }
        }

        Flats {
            wad_file: Rc::clone(wad_file),
            map: HashMap::new(),
            animated_flats,
        }
    }

    pub fn get(&mut self, name: &str) -> Rc<Flat> {
        if !self.map.contains_key(name) {
            // Create the flat & insert it
            self.map
                .insert(name.to_string(), Rc::new(Flat::new(&self.wad_file, name)));
        }

        Rc::clone(self.map.get(name).unwrap())
    }

    // Get a texture which may be animated
    pub fn get_animated(&mut self, name: &str, timestamp: f32) -> Rc<Flat> {
        if let Some(list) = self.animated_flats.get(name) {
            // Cycle 3 times a second
            let cycle = ((timestamp - f32::trunc(timestamp)) * 3.0) as usize;
            self.get(&list[cycle].clone())
        } else {
            self.get(name)
        }
    }
}

impl Flat {
    // Create a new flat and load the pixels
    pub fn new(wad_file: &WadFile, name: &str) -> Flat {
        let dir_entry = wad_file.get_dir_entry(name).unwrap();
        let offset = dir_entry.offset as usize;
        let wad_file = &wad_file;

        let mut pixels: Vec<Vec<u8>> = Vec::with_capacity(FLAT_SIZE as usize);
        for _ in 0..FLAT_SIZE as usize {
            let mut row = Vec::new();
            row.resize(FLAT_SIZE as usize, 0u8);
            pixels.push(row);
        }

        for y in 0..FLAT_SIZE as usize {
            for x in 0..FLAT_SIZE as usize {
                pixels[y][x] = wad_file.file[offset + y * FLAT_SIZE as usize + x];
            }
        }

        Flat {
            name: name.to_string(),
            pixels,
        }
    }

    // Draw the flat to the top-left corner
    #[allow(dead_code)]
    pub fn test_flat_draw(&self, game: &mut Game) {
        for x in 0..FLAT_SIZE as usize {
            for y in 0..FLAT_SIZE as usize {
                let value = self.pixels[y][x];
                let color = game.palette.colors[value as usize];
                game.canvas.set_draw_color(color);
                let rect = Rect::new(x as i32 * 4, y as i32 * 4, 4, 4);
                game.canvas.fill_rect(rect).unwrap();
            }
        }
    }
}

impl fmt::Debug for Flat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Flat: {}", self.name,)
    }
}
