use crate::wad::{MapLumpName, WadFile};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
}

pub fn load_vertexes(wad_file: &WadFile, map_name: &str) -> Vec<Vertex> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Vertexes);
    let count = dir_entry.size as usize / 4; // A vertex is 4 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 4;
        let vertex = Vertex {
            x: wad_file.read_i16(offset),
            y: wad_file.read_i16(offset + 2),
        };
        results.push(vertex);
    }

    results
}
