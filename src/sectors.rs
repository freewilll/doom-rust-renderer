use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::wad::{MapLumpName, WadFile};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Sector {
    pub id: i16,
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_texture: String,
    pub floor_texture_hash: i16, // Until texture rendering is implemented, add a unique id for a texture
    pub ceiling_texture: String,
    pub ceiling_texture_hash: i16, // Until texture rendering is implemented, add a unique id for a texture
    pub light_level: i16,
    pub special_type: i16,
    pub tag_number: i16,
}

fn make_hash<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

pub fn load_sectors(wad_file: &WadFile, map_name: &str) -> Vec<Rc<Sector>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Sectors);
    let count = dir_entry.size as usize / 26; // A sector is 26 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 26;

        let floor_texture = wad_file.read_lump_name(offset + 4);
        let floor_texture_hash = make_hash(&floor_texture) as i16;

        let ceiling_texture = wad_file.read_lump_name(offset + 12);
        let ceiling_texture_hash = make_hash(&ceiling_texture) as i16;

        let sector = Sector {
            id: i as i16,
            floor_height: wad_file.read_i16(offset),
            ceiling_height: wad_file.read_i16(offset + 2),
            floor_texture,
            floor_texture_hash,
            ceiling_texture,
            ceiling_texture_hash,
            light_level: wad_file.read_i16(offset + 20),
            special_type: wad_file.read_i16(offset + 22),
            tag_number: wad_file.read_i16(offset + 24),
        };
        results.push(Rc::new(sector));
    }

    results
}
