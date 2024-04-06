use std::cell::RefCell;
use std::rc::Rc;

use crate::wad::{MapLumpName, WadFile};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Sector {
    pub id: i16,
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_texture: String,
    pub ceiling_texture: String,
    pub light_level: i16,
    pub special_type: i16,
    pub tag_number: i16,
}

pub fn load_sectors(wad_file: &WadFile, map_name: &str) -> Vec<Rc<RefCell<Sector>>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Sectors);
    let count = dir_entry.size as usize / 26; // A sector is 26 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 26;

        let floor_texture = wad_file.read_lump_name(offset + 4);
        let ceiling_texture = wad_file.read_lump_name(offset + 12);

        let sector = Sector {
            id: i as i16,
            floor_height: wad_file.read_i16(offset),
            ceiling_height: wad_file.read_i16(offset + 2),
            floor_texture,
            ceiling_texture,
            light_level: wad_file.read_i16(offset + 20),
            special_type: wad_file.read_i16(offset + 22),
            tag_number: wad_file.read_i16(offset + 24),
        };
        results.push(Rc::new(RefCell::new(sector)));
    }

    results
}
