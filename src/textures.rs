use sdl2::rect::Rect;
use std::collections::HashMap;
use std::rc::Rc;
use std::{fmt, str};

use crate::game::Game;
use crate::pictures::Picture;
use crate::wad::{DirEntry, WadFile};

// A texture consists of a list of patches. Each patch has an origin (x,y) and refers to
// an entry in the PNAMES lump which has as lump name of a picture.
// Textures are lazy-loaded. On first load, all the patch pictures are loaded
// and the pixels 2-D vec is created from it.

// Names of wall patches + offsets into the WAD file
#[derive(Debug)]
pub struct Pname {
    pub name: String,            // Lump name
    pub wad_offset: Option<u32>, // Offset in WAD file (None if the lump doesn't exist)
}

// Patch is a lazy loaded picture + offset within the texture
struct Patch {
    origin_x: i16,                // The horizontal offset relative to the upper-left
    origin_y: i16,                // The vertical offset relative to the upper-left
    patch_number: i16,            // The patch number (as listed in PNAMES) to draw
    picture: Option<Rc<Picture>>, // A lazy loaded reference collected picture
    wad_file: Rc<WadFile>,        // Needed to be able to lazy load textures
}

// A texture definition contains the data needed to load a texture. It's data comes
// straight from the WAD file.
pub struct TextureDefinition {
    width: i16,
    height: i16,
    patches: Vec<Patch>,
    texture: Option<Rc<Texture>>, // The loaded texture
}

// A Texture is a loaded texture, with its pixels populated from the patches
pub struct Texture {
    pub width: i16,
    pub height: i16,
    pub pixels: Vec<Vec<u8>>, // Grid of colormap indexes
}

// A struct to handle lazy loaded textures
pub struct Textures {
    definitions: HashMap<String, TextureDefinition>, // The available textures
    wad_file: Rc<WadFile>,                           // Needed to be able to lazy load textures
    pub pnames: Vec<Pname>,                          // Parsed contents of the PNAMES lump
}

impl Patch {
    // Lazy load the picture if not already done
    pub fn get_picture(&mut self, pnames: &Vec<Pname>) -> Rc<Picture> {
        if let Some(picture) = &self.picture {
            return Rc::clone(&picture);
        };

        let patch_name = &pnames[self.patch_number as usize].name;
        let rc_picture = Rc::new(Picture::new(&self.wad_file, patch_name));
        self.picture = Some(Rc::clone(&rc_picture));

        rc_picture
    }
}

impl Texture {
    // Load a texture by first loading all the patches, then setting
    // the pixels from the patches.
    fn load(&mut self, definition: &mut TextureDefinition, pnames: &Vec<Pname>) {
        self.pixels = Vec::with_capacity(self.height as usize);
        for _ in 0..self.height as usize {
            let mut arr = Vec::new();
            arr.resize(self.width as usize, 0u8);
            self.pixels.push(arr);
        }

        for patch in &mut definition.patches {
            let picture = patch.get_picture(pnames);

            for x in 0..picture.width as usize {
                for y in 0..picture.height as usize {
                    let value = picture.pixels[y][x];

                    let picture_x = x as i16 + patch.origin_x;
                    let picture_y = y as i16 + patch.origin_y;

                    if picture_x >= 0
                        && picture_x < self.width
                        && picture_y >= 0
                        && picture_y < self.height
                    {
                        self.pixels[(y as i16 + patch.origin_y) as usize]
                            [(x as i16 + patch.origin_x) as usize] = value;
                    }
                }
            }
        }
    }

    // Draw the picture to the top-left corner
    #[allow(dead_code)]
    pub fn test_flat_draw(&self, game: &mut Game) {
        for x in 0..self.width as usize {
            for y in 0..self.height as usize {
                let value = self.pixels[y][x];
                let color = game.palette.colors[value as usize];
                game.canvas.set_draw_color(color);
                let rect = Rect::new(x as i32 * 4, y as i32 * 4, 4, 4);
                game.canvas.fill_rect(rect).unwrap();
            }
        }
    }
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Texture: dimensions: {} x {}", self.width, self.height)
    }
}

impl Textures {
    pub fn new(wad_file: &Rc<WadFile>) -> Textures {
        let mut textures = Textures {
            wad_file: Rc::clone(wad_file),
            definitions: HashMap::new(),
            pnames: Vec::new(),
        };

        textures.load_pnames();

        // TEXTURE1 is always present
        let texture1_dir_entry = wad_file.get_dir_entry("TEXTURE1").unwrap();
        textures.load_texture_list(texture1_dir_entry);

        // TEXTURE2 is only present in the registered version of Doom 1
        if let Ok(dir_entry) = wad_file.get_dir_entry("TEXTURE2") {
            textures.load_texture_list(&dir_entry);
        }

        textures
    }

    // Return a texture from the cache, otherwise load it
    pub fn get(&mut self, name: &str) -> Rc<Texture> {
        let definition: &mut TextureDefinition = self
            .definitions
            .get_mut(&name.to_ascii_uppercase())
            .unwrap_or_else(|| panic!("Unknown texture {}", &name));

        // Already loaded
        if let Some(texture) = &definition.texture {
            return Rc::clone(&texture);
        }

        // Load the texture
        let mut texture = Texture {
            width: definition.width,
            height: definition.height,
            pixels: Vec::new(),
        };

        texture.load(definition, &self.pnames);

        let rc_texture = Rc::new(texture);
        definition.texture = Some(Rc::clone(&rc_texture));

        Rc::clone(&rc_texture)
    }

    // Load and parse PNAMES section. Look up the lump names in the WAD file.
    fn load_pnames(&mut self) {
        let wad_file = &self.wad_file;

        let pnames_dir_entry = wad_file.get_dir_entry("PNAMES").unwrap();

        let offset = pnames_dir_entry.offset as usize;
        let count = wad_file.read_u32(offset);

        let mut pnames: Vec<Pname> = Vec::new();
        for i in 0..count as usize {
            let name = wad_file.read_lump_name(offset + 4 + i * 8);
            let dir_entry = wad_file.get_dir_entry(&name);
            let wad_offset = if let Ok(dir_entry) = dir_entry {
                Some(dir_entry.offset)
            } else {
                None
            };

            pnames.push(Pname { name, wad_offset });
        }

        self.pnames = pnames;
    }

    // Load TEXTURE1 or TEXTURE2 lump. This contains names of all the textures + patches they
    // are made up of.
    fn load_texture_list(&mut self, dir_entry: &DirEntry) {
        let wad_file = &self.wad_file;

        let texture_list_offset = dir_entry.offset as usize;

        let texture_count = wad_file.read_u32(texture_list_offset);

        for i in 0..texture_count as usize {
            let map_texture_offset = wad_file.read_u32(texture_list_offset + 4 + 4 * i) as usize;
            let offset = texture_list_offset + map_texture_offset;
            let name = wad_file.read_lump_name(offset);

            let width = wad_file.read_i16(offset + 12);
            let height = wad_file.read_i16(offset + 14);
            let patch_count = wad_file.read_i16(offset + 20);

            let patch0_offset = offset + 22;
            let mut patches: Vec<Patch> = Vec::new();

            for j in 0..patch_count as usize {
                let patch_offset = patch0_offset + j * 10;

                let origin_x = wad_file.read_i16(patch_offset + 0);
                let origin_y = wad_file.read_i16(patch_offset + 2);
                let patch_number = wad_file.read_i16(patch_offset + 4);

                let patch = Patch {
                    origin_x,
                    origin_y,
                    patch_number,
                    picture: None,
                    wad_file: Rc::clone(&wad_file),
                };

                patches.push(patch);
            }

            let texture_definition = TextureDefinition {
                width,
                height,
                patches,
                texture: None,
            };

            self.definitions
                .insert(name.to_ascii_uppercase(), texture_definition);
        }
    }
}
