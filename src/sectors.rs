use crate::wad::{MapLumpName, WadFile};
use std::rc::Rc;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Sector {
    floor_height: i16,
    ceiling_height: i16,
    floor_texture: String,
    ceiling_texture: String,
    light_level: i16,
    special_type: i16,
    tag_number: i16,
}

pub fn load_sectors(wad_file: &WadFile, map_name: &str) -> Vec<Rc<Sector>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Sectors);
    let count = dir_entry.size as usize / 26; // A sector is 26 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 26;

        let sector = Sector {
            floor_height: wad_file.read_i16(offset),
            ceiling_height: wad_file.read_i16(offset + 2),
            floor_texture: wad_file.read_lump_name(offset + 4),
            ceiling_texture: wad_file.read_lump_name(offset + 12),
            light_level: wad_file.read_i16(offset + 20),
            special_type: wad_file.read_i16(offset + 22),
            tag_number: wad_file.read_i16(offset + 24),
        };
        results.push(Rc::new(sector));
    }

    results
}
