use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::rc::Rc;
use std::{fmt, str};

use crate::bitmap::Bitmap;
use crate::map::Vertex;
use crate::palette::Palette;
use crate::wad::WadFile;

// Lazy loaded hashmap of pictures
#[allow(dead_code)]
pub struct Pictures {
    map: HashMap<String, Rc<Picture>>, // The reference counted pictures
    wad_file: Rc<WadFile>,             // Needed to be able to lazy load the pictures
}

// A picture (aka patch)
#[allow(dead_code)]
#[derive(Clone)]
pub struct Picture {
    pub name: String,       // The name
    wad_offset: u32,        // Offset in the WAD file
    pub bitmap: Rc<Bitmap>, // Bitmap
    pub left_offset: i16,   // Offset in pixels to the left of the origin
    pub top_offset: i16,    // Offset in pixels below the origin
}

impl Pictures {
    pub fn new(wad_file: &Rc<WadFile>) -> Pictures {
        Pictures {
            wad_file: Rc::clone(wad_file),
            map: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get(&mut self, name: &str) -> Result<Rc<Picture>, String> {
        if !self.map.contains_key(name) {
            let picture = Picture::new(&self.wad_file, name)?;

            // Create the picture & insert it
            self.map.insert(name.to_string(), Rc::new(picture));
        }

        Ok(Rc::clone(self.map.get(name).unwrap()))
    }

    #[allow(dead_code)]
    pub fn test_draw(
        &mut self,
        canvas: &mut Canvas<Window>,
        palette: &Palette,
        name: &str,
        offset: &Vertex,
    ) {
        self.get(name)
            .unwrap()
            .bitmap
            .test_flat_draw(canvas, palette, offset);
    }
}

impl Picture {
    // Create a new picture and load the pixels
    pub fn new(wad_file: &WadFile, name: &str) -> Result<Picture, String> {
        let dir_entry = wad_file.get_dir_entry(name)?;
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

        let mut bitmap = Bitmap::new(width, height, pixels);

        Self::read_pixels(wad_file, dir_entry.offset, &mut bitmap);

        let picture = Picture {
            name: name.to_string(),
            wad_offset: dir_entry.offset,
            bitmap: Rc::new(bitmap),
            left_offset,
            top_offset,
        };

        Ok(picture)
    }

    // https://doomwiki.org/wiki/Picture_format
    // Decode a "picture format" lump
    pub fn read_pixels(wad_file: &WadFile, wad_offset: u32, bitmap: &mut Bitmap) {
        // Loop over columns
        for column in 0..bitmap.width as usize {
            let mut column_offset = wad_offset as usize
                + wad_file.read_u32(wad_offset as usize + column * 4 + 8) as usize;

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

                    bitmap.pixels[y as usize][x as usize] = Some(value);
                }

                column_offset += length as usize + 4;
            }
        }
    }

    // Create new picture with a mirror image of the picture
    pub fn mirror(&self) -> Picture {
        let mut bitmap = (*self.bitmap).clone();

        for y in 0..bitmap.height as usize {
            let row = &mut bitmap.pixels[y];
            for x in 0..bitmap.width as usize / 2_usize {
                (row[x], row[bitmap.width as usize - 1 - x]) =
                    (row[bitmap.width as usize - 1 - x], row[x]);
            }
        }

        Picture {
            name: self.name.clone(),
            wad_offset: self.wad_offset,
            bitmap: Rc::new(bitmap),
            left_offset: self.left_offset,
            top_offset: self.top_offset,
        }
    }
}

impl fmt::Debug for Picture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Picture: bitmap: {:?}, left_offset: {}, top_offset: {}",
            self.bitmap, self.left_offset, self.top_offset,
        )
    }
}
