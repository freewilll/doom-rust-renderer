use crate::wad::{MapLumpName, WadFile};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Linedef {
    pub start_vertex: i16,
    pub end_vertex: i16,
    pub flags: i16,
    pub special_type: i16,
    pub sector_tag: i16,
    pub front_sidedef: i16,
    pub back_sidedef: i16,
}

pub fn load_linedefs(wad_file: &WadFile, map_name: &str) -> Vec<Linedef> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Linedefs);
    let count = dir_entry.size as usize / 14; // A linedef is 14 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 14;
        let linedef = Linedef {
            start_vertex: wad_file.read_i16(offset),
            end_vertex: wad_file.read_i16(offset + 2),
            flags: 0,
            special_type: 0,
            sector_tag: 0,
            front_sidedef: 0,
            back_sidedef: 0,
        };
        results.push(linedef);
    }

    results
}
