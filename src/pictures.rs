use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::rc::Rc;
use std::{fmt, str};

use crate::game::Game;
use crate::wad::WadFile;

// Lazy loaded hashmap of pictures
#[allow(dead_code)]
pub struct Pictures {
    map: HashMap<String, Rc<Picture>>, // The reference counted pictures
    wad_file: Rc<WadFile>,             // Needed to be able to lazy load the pictures
}

// A picture (aka patch)
#[allow(dead_code)]
pub struct Picture {
    pub name: String,                 // The name
    wad_offset: u32,                  // Offset in the WAD file
    pub width: i16,                   // Width of graphic
    pub height: i16,                  // Height of graphic
    pub left_offset: i16,             // Offset in pixels to the left of the origin
    pub top_offset: i16,              // Offset in pixels below the origin
    pub pixels: Vec<Vec<Option<u8>>>, // Grid of colormap indexes or None if transparent
}

impl Pictures {
    pub fn new(wad_file: &Rc<WadFile>) -> Pictures {
        Pictures {
            wad_file: Rc::clone(wad_file),
            map: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get(&mut self, name: &str) -> Rc<Picture> {
        if !self.map.contains_key(name) {
            // Create the picture & insert it
            self.map.insert(
                name.to_string(),
                Rc::new(Picture::new(&self.wad_file, name)),
            );
        }

        Rc::clone(self.map.get(name).unwrap())
    }
}

impl Picture {
    // Create a new picture and load the pixels
    pub fn new(wad_file: &WadFile, name: &str) -> Picture {
        let dir_entry = wad_file.get_dir_entry(name).unwrap();
        let offset = dir_entry.offset as usize;
        let wad_file = &wad_file;

        let width = wad_file.read_i16(offset);
        let height = wad_file.read_i16(offset + 2);
        let left_offset = wad_file.read_i16(offset + 4);
        let top_offset = wad_file.read_i16(offset + 6);

        let mut pixels: Vec<Vec<Option<u8>>> = Vec::with_capacity(height as usize);
        for _ in 0..height as usize {
            let mut row = Vec::new();
            row.resize(width as usize, None);
            pixels.push(row);
        }

        let mut picture = Picture {
            name: name.to_string(),
            wad_offset: dir_entry.offset,
            width,
            height,
            left_offset,
            top_offset,
            pixels,
        };

        picture.read_pixels(wad_file);

        picture
    }

    // https://doomwiki.org/wiki/Picture_format
    // Decode a "picture format" lump
    pub fn read_pixels(&mut self, wad_file: &WadFile) {
        // Loop over columns
        for column in 0..self.width as usize {
            let mut column_offset = self.wad_offset as usize
                + wad_file.read_u32(self.wad_offset as usize + column * 4 + 8) as usize;

            // Loop over posts
            loop {
                let y_offset = wad_file.file[column_offset];
                if y_offset == 0xff {
                    break;
                }

                let length = wad_file.file[column_offset + 1];

                for row in 0..length as usize {
                    let value = wad_file.file[column_offset + row + 3];
                    let x = column as i32;
                    let y = row as i32 + y_offset as i32;

                    self.pixels[y as usize][x as usize] = Some(value);
                }

                column_offset += length as usize + 4;
            }
        }
    }

    // Draw the picture to the top-left corner
    #[allow(dead_code)]
    pub fn test_flat_draw(&self, game: &mut Game) {
        game.canvas.set_draw_color(Color::RGB(0, 255, 255));
        let rect = Rect::new(0, 0, self.width as u32 * 4, self.height as u32 * 4);
        game.canvas.fill_rect(rect).unwrap();

        for x in 0..self.width as usize {
            for y in 0..self.height as usize {
                if let Some(value) = self.pixels[y][x] {
                    let color = game.palette.colors[value as usize];
                    game.canvas.set_draw_color(color);
                    let rect = Rect::new(x as i32 * 4, y as i32 * 4, 4, 4);
                    game.canvas.fill_rect(rect).unwrap();
                }
            }
        }
    }
}

impl fmt::Debug for Picture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Picture: dimensions: {} x {}, left_offset: {}, top_offset: {}",
            self.width, self.height, self.left_offset, self.top_offset,
        )
    }
}
