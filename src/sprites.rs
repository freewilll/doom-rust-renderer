use std::collections::HashMap;
use std::rc::Rc;

use crate::bitmap::Bitmap;
use crate::info::{SpriteId, SPRITES};
use crate::pictures::Pictures;
use crate::wad::WadFile;

pub struct Sprites {
    map: HashMap<SpriteId, Sprite>,
}

// A map from frame to SpriteFrame
pub struct Sprite {
    frames: HashMap<u8, SpriteFrame>,
}

#[derive(Clone)]
#[allow(dead_code)]
// One bitmap for each rotation, or a single bitmap for non-rotated sprites
pub struct SpriteFrame {
    rotate: bool,             // Is it rotated?
    bitmaps: Vec<Rc<Bitmap>>, // one or eight bitmaps
}

impl Sprites {
    pub fn new(wad_file: &WadFile, pictures: &mut Pictures) -> Sprites {
        let mut map: HashMap<SpriteId, Sprite> = HashMap::new();

        for sprite_id in SPRITES {
            let sprite_name = format!("{:?}", sprite_id);

            // Indexed on frame, rotation
            let mut found_sprites: HashMap<u8, HashMap<u8, Rc<Bitmap>>> = HashMap::new();

            for index in wad_file.first_sprite_lump..wad_file.last_sprite_lump {
                let dir_entry = &wad_file.dirs_list[index as usize];
                if dir_entry.name.starts_with(&sprite_name) {
                    let picture = pictures.get(&dir_entry.name).unwrap();
                    let bitmap = Rc::clone(&picture.bitmap);

                    let frame = dir_entry.name.as_bytes()[4] - 65;
                    let rotation = dir_entry.name.as_bytes()[5] - 48;

                    found_sprites
                        .entry(frame)
                        .or_insert_with(|| HashMap::new())
                        .insert(rotation, Rc::clone(&bitmap));

                    if dir_entry.name.len() > 6 {
                        let frame = dir_entry.name.as_bytes()[6] - 65;
                        let rotation = dir_entry.name.as_bytes()[7] - 48;

                        found_sprites
                            .entry(frame)
                            .or_insert_with(|| HashMap::new())
                            .insert(rotation, Rc::new(bitmap.mirror()));
                    }
                }
            }

            let mut sprite = Sprite {
                frames: HashMap::new(),
            };

            for (frame, rotations) in found_sprites.iter() {
                let rotate = rotations.keys().len() != 1;

                let mut sprite_frame = SpriteFrame {
                    rotate: rotate,
                    bitmaps: Vec::with_capacity(8),
                };

                if rotate {
                    if rotations.keys().len() != 8 {
                        panic!(
                            "Got something other than 8 rotations for {}/{}: {}",
                            sprite_name,
                            frame,
                            rotations.keys().len()
                        );
                    }

                    for rotation in 1..9 as u8 {
                        let sprite_frame_bitmap = rotations.get(&rotation).unwrap();
                        sprite_frame.bitmaps.push(sprite_frame_bitmap.clone());
                    }
                } else {
                    let sprite_frame_bitmap = rotations.get(&0u8).unwrap();
                    sprite_frame.bitmaps.push(sprite_frame_bitmap.clone());
                }
                sprite.frames.insert(*frame, sprite_frame);
            }

            map.insert(sprite_id, sprite);
        }

        Sprites { map: map }
    }

    pub fn get_bitmap(&self, sprite_id: &SpriteId, frame_id: u8, rotation: u8) -> Rc<Bitmap> {
        let sprite = self.map.get(&sprite_id).unwrap();
        let frame = sprite
            .frames
            .get(&frame_id)
            .unwrap_or_else(|| panic!("Unknown frame {} for {:?}", frame_id, sprite_id));

        if rotation > 7 {
            panic!("Invalid rotation {}", rotation);
        }

        let sprite_frame_bitmap = if frame.rotate {
            &frame.bitmaps[rotation as usize]
        } else {
            &frame.bitmaps[0]
        };

        Rc::clone(&sprite_frame_bitmap)
    }
}
