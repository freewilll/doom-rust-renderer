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
}

// A 64x64 pixel flat
pub struct Flat {
    pub name: String,
    pub pixels: Vec<Vec<u8>>, // Grid of colormap indexes
}

impl Flats {
    pub fn new(wad_file: &Rc<WadFile>) -> Flats {
        Flats {
            wad_file: Rc::clone(wad_file),
            map: HashMap::new(),
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
}

impl Flat {
    // Create a new flat and load the pixels
    pub fn new(wad_file: &WadFile, name: &str) -> Flat {
        let dir_entry = wad_file.get_dir_entry(name).unwrap();
        let offset = dir_entry.offset as usize;
        let wad_file = &wad_file;

        let mut pixels: Vec<Vec<u8>> = Vec::with_capacity(FLAT_SIZE as usize);
        for _ in 0..FLAT_SIZE as usize {
            let mut arr = Vec::new();
            arr.resize(FLAT_SIZE as usize, 0u8);
            pixels.push(arr);
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
