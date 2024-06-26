use crate::geometry::BoundingBox;
use crate::map::SubSector;
use crate::wad::{MapLumpName, WadFile};
use std::rc::Rc;

const NODE_IS_SUBSECTOR: i16 = 1 << 15;

// A node's child is either a node itself or a subsector
#[derive(Debug)]
pub enum NodeChild {
    Node(Rc<Node>),
    SubSector(Rc<SubSector>),
}

impl NodeChild {
    // Create a NodeChild which is either a node or subsector from an index in the WAD file
    fn from_index(index: i16, nodes: &[Rc<Node>], subsectors: &[Rc<SubSector>]) -> NodeChild {
        let is_subsector = index & NODE_IS_SUBSECTOR == NODE_IS_SUBSECTOR;
        let stripped_index = (index & !NODE_IS_SUBSECTOR) as usize;

        if is_subsector {
            NodeChild::SubSector(Rc::clone(&subsectors[stripped_index]))
        } else {
            NodeChild::Node(Rc::clone(&nodes[stripped_index]))
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Node {
    pub x: f32,                          // x coordinate of partition line start
    pub y: f32,                          // x coordinate of partition line start
    pub dx: f32,                         // Change in x from start to end of partition line
    pub dy: f32,                         // Change in y from start to end of partition line
    pub right_bounding_box: BoundingBox, // Right bounding box
    pub left_bounding_box: BoundingBox,  // Left bounding box
    pub right_child: NodeChild,
    pub left_child: NodeChild,
}

// Load the node tree. Nodes in the WAD file are in order from bottom up, so
// a  child node indexes are always lower than a node index. Conveniently,
// the node tree can be built in one pass. The last node is the root node.
pub fn load_nodes(
    wad_file: &WadFile,
    subsectors: &[Rc<SubSector>],
    map_name: &str,
) -> Vec<Rc<Node>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Nodes);
    let count = dir_entry.size as usize / 28; // A node is 28 bytes long

    let mut nodes = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 28;

        let node = Node {
            x: wad_file.read_f32_from_i16(offset),
            y: wad_file.read_f32_from_i16(offset + 2),
            dx: wad_file.read_f32_from_i16(offset + 4),
            dy: wad_file.read_f32_from_i16(offset + 6),

            right_bounding_box: BoundingBox {
                top: wad_file.read_f32_from_i16(offset + 8),
                bottom: wad_file.read_f32_from_i16(offset + 10),
                left: wad_file.read_f32_from_i16(offset + 12),
                right: wad_file.read_f32_from_i16(offset + 14),
            },
            left_bounding_box: BoundingBox {
                top: wad_file.read_f32_from_i16(offset + 16),
                bottom: wad_file.read_f32_from_i16(offset + 18),
                left: wad_file.read_f32_from_i16(offset + 20),
                right: wad_file.read_f32_from_i16(offset + 22),
            },

            right_child: NodeChild::from_index(wad_file.read_i16(offset + 24), &nodes, subsectors),
            left_child: NodeChild::from_index(wad_file.read_i16(offset + 26), &nodes, subsectors),
        };
        nodes.push(Rc::new(node));
    }

    nodes
}
