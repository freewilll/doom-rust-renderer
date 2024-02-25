use crate::wad::{MapLumpName, WadFile};

#[derive(Debug)]
#[allow(dead_code)]
pub struct SubSector {
    pub seg_count: i16,
    pub first_seg_number: i16,
}

pub fn load_subsectors(wad_file: &WadFile, map_name: &str) -> Vec<SubSector> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Ssectors);
    let count = dir_entry.size as usize / 4; // A subsector is 4 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 4;
        let subsector = SubSector {
            seg_count: wad_file.read_i16(offset),
            first_seg_number: wad_file.read_i16(offset + 2),
        };
        results.push(subsector);
    }

    results
}
