use std::cell::RefCell;
use std::rc::Rc;

use crate::map::Sector;
use crate::wad::{MapLumpName, WadFile};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Sidedef {
    pub id: i16,
    pub x_offset: f32,
    pub y_offset: f32,
    pub upper_texture: String,
    pub lower_texture: String,
    pub middle_texture: String,
    pub sector: Rc<RefCell<Sector>>, // Sector number this sidedef 'faces'
}

pub fn load_sidedefs(
    wad_file: &WadFile,
    sectors: &[Rc<RefCell<Sector>>],
    map_name: &str,
) -> Vec<Rc<Sidedef>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Sidedefs);
    let count = dir_entry.size as usize / 30; // A sidedef is 30 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 30;

        let sidedef = Sidedef {
            id: i as i16,
            x_offset: wad_file.read_f32_from_i16(offset),
            y_offset: wad_file.read_f32_from_i16(offset + 2),
            upper_texture: wad_file.read_lump_name(offset + 4),
            lower_texture: wad_file.read_lump_name(offset + 12),
            middle_texture: wad_file.read_lump_name(offset + 20),
            sector: Rc::clone(&sectors[wad_file.read_i16(offset + 28) as usize]),
        };
        results.push(Rc::new(sidedef));
    }

    results
}
