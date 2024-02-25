use crate::wad::{MapLumpName, WadFile};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Seg {
    pub start_vertex: i16,
    pub end_vertex: i16,
    pub angle: i16,
    pub linedef_number: i16,
    pub direction: bool, // 0 (same as linedef) or 1 (opposite of linedef)
    pub offset: i16,     // distance along linedef to start of seg
}

pub fn load_segs(wad_file: &WadFile, map_name: &str) -> Vec<Seg> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Segs);
    let count = dir_entry.size as usize / 12; // A seg is 12 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 12;
        let sef = Seg {
            start_vertex: wad_file.read_i16(offset),
            end_vertex: wad_file.read_i16(offset + 2),
            angle: wad_file.read_i16(offset + 4),
            linedef_number: wad_file.read_i16(offset + 6),
            direction: wad_file.read_i16(offset + 8) != 0,
            offset: wad_file.read_i16(offset + 10),
        };
        results.push(sef);
    }

    results
}
