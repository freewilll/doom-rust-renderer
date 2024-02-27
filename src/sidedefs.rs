use crate::sectors::Sector;
use crate::wad::{MapLumpName, WadFile};
use std::rc::Rc;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Sidedef {
    x_offset: f32,
    y_offset: f32,
    upper_texture: String,
    lower_texture: String,
    middle_texture: String,
    sector: Rc<Sector>, // Sector number this sidedef 'faces'
}

pub fn load_sidedefs(
    wad_file: &WadFile,
    sectors: &Vec<Rc<Sector>>,
    map_name: &str,
) -> Vec<Rc<Sidedef>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Sidedefs);
    let count = dir_entry.size as usize / 30; // A sidedef is 30 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 30;

        let sidedef = Sidedef {
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
