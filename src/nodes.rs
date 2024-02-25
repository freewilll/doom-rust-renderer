use crate::geometry::BoundingBox;
use crate::wad::{MapLumpName, WadFile};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Node {
    pub x: i16,                      // x coordinate of partition line start
    pub y: i16,                      // x coordinate of partition line start
    pub dx: i16,                     // Change in x from start to end of partition line
    pub dy: i16,                     // Change in y from start to end of partition line
    right_bounding_box: BoundingBox, // Right bounding box
    left_bounding_box: BoundingBox,  // Left bounding box
    right_child: i16,                // Right child node
    left_child: i16,                 // Left child node
}

pub fn load_nodes(wad_file: &WadFile, map_name: &str) -> Vec<Node> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Nodes);
    let count = dir_entry.size as usize / 28; // A node is 28 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 28;
        let node = Node {
            x: wad_file.read_i16(offset + 0),
            y: wad_file.read_i16(offset + 2),
            dx: wad_file.read_i16(offset + 4),
            dy: wad_file.read_i16(offset + 6),
            right_bounding_box: BoundingBox {
                top: wad_file.read_i16(offset + 8),
                bottom: wad_file.read_i16(offset + 10),
                left: wad_file.read_i16(offset + 10),
                right: wad_file.read_i16(offset + 12),
            },
            left_bounding_box: BoundingBox {
                top: wad_file.read_i16(offset + 16),
                bottom: wad_file.read_i16(offset + 18),
                left: wad_file.read_i16(offset + 20),
                right: wad_file.read_i16(offset + 22),
            },
            right_child: wad_file.read_i16(offset + 24),
            left_child: wad_file.read_i16(offset + 26),
        };
        results.push(node);
    }

    results
}
