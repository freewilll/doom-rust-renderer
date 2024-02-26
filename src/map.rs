use crate::geometry::BoundingBox;
use crate::linedefs::{load_linedefs, Linedef};
use crate::nodes::{load_nodes, Node};
use crate::segs::{load_segs, Seg};
use crate::subsectors::{load_subsectors, SubSector};
use crate::vertexes::{load_vertexes, Vertex};
use crate::wad::WadFile;

#[allow(dead_code)]
pub struct Map {
    pub vertexes: Vec<Vertex>,      // Vertexes that make up the lines
    pub linedefs: Vec<Linedef>,     // Lines
    pub segs: Vec<Seg>,             // Lines, split by the BSP builder
    pub subsectors: Vec<SubSector>, // Sectors, split by the BSP builder
    pub nodes: Vec<Node>,           // BSP tree
    pub bounding_box: BoundingBox,  // Bounding box for the whole map
}

impl Map {
    // Load map
    pub fn new(wad_file: &WadFile, map_name: &str) -> Map {
        let vertexes = load_vertexes(wad_file, map_name);
        let linedefs = load_linedefs(wad_file, map_name);
        let segs = load_segs(wad_file, map_name);
        let subsectors = load_subsectors(wad_file, map_name);
        let nodes = load_nodes(wad_file, map_name);

        let mut bounding_box = BoundingBox::extendable_new();

        for linedef in &linedefs {
            let start_vertex = &vertexes[linedef.start_vertex as usize];
            let end_vertex = &vertexes[linedef.end_vertex as usize];

            bounding_box.extend(&start_vertex);
            bounding_box.extend(&end_vertex);
        }

        Map {
            vertexes: vertexes,
            linedefs: linedefs,
            segs: segs,
            subsectors: subsectors,
            nodes: nodes,
            bounding_box: bounding_box,
        }
    }
}
