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
            x: i16::from_le_bytes(wad_file.file[offset..offset + 2].try_into().unwrap()),
            y: i16::from_le_bytes(wad_file.file[offset + 2..offset + 4].try_into().unwrap()),
        };
        results.push(vertex);
    }

    results
}
