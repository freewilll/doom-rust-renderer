use crate::geometry::BoundingBox;
use crate::linedefs::{load_linedefs, Linedef};
use crate::nodes::{load_nodes, Node};
use crate::segs::{load_segs, Seg};
use crate::subsectors::{load_subsectors, SubSector};
use crate::vertexes::{load_vertexes, Vertex};
use crate::wad::WadFile;
use std::rc::Rc;

#[allow(dead_code)]
pub struct Map {
    pub vertexes: Vec<Rc<Vertex>>,      // Vertexes that make up the lines
    pub linedefs: Vec<Rc<Linedef>>,     // Lines
    pub segs: Vec<Rc<Seg>>,             // Lines, split by the BSP builder
    pub subsectors: Vec<Rc<SubSector>>, // Sectors, split by the BSP builder
    pub nodes: Vec<Rc<Node>>,           // BSP tree
    pub root_node: Rc<Node>,            // Root node of the BSP tree
    pub bounding_box: BoundingBox,      // Bounding box for the whole map
}

impl Map {
    // Load map
    pub fn new(wad_file: &WadFile, map_name: &str) -> Map {
        let vertexes = load_vertexes(wad_file, map_name);
        let linedefs = load_linedefs(wad_file, &vertexes, map_name);
        let segs = load_segs(wad_file, &vertexes, &linedefs, map_name);
        let subsectors = load_subsectors(wad_file, &segs, map_name);
        let nodes = load_nodes(wad_file, &subsectors, map_name);
        let root_node = Rc::clone(&nodes[nodes.len() - 1]);

        let mut bounding_box = BoundingBox::extendable_new();

        for linedef in &linedefs {
            bounding_box.extend(&linedef.start_vertex);
            bounding_box.extend(&linedef.end_vertex);
        }

        Map {
            vertexes: vertexes,
            linedefs: linedefs,
            segs: segs,
            subsectors: subsectors,
            nodes: nodes,
            root_node: root_node,
            bounding_box: bounding_box,
        }
    }
}
