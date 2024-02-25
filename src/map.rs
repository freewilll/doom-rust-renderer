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
    pub top_left: Vertex,           // Top left vertex of the map
    pub bottom_right: Vertex,       // Bottom right vertex of the map
}

impl Map {
    // Load map
    pub fn new(wad_file: &WadFile, map_name: &str) -> Map {
        let mut min_x = i16::MAX;
        let mut max_x = i16::MIN;
        let mut min_y = i16::MAX;
        let mut max_y = i16::MIN;

        let vertexes = load_vertexes(wad_file, map_name);
        let linedefs = load_linedefs(wad_file, map_name);
        let segs = load_segs(wad_file, map_name);
        let subsectors = load_subsectors(wad_file, map_name);
        let nodes = load_nodes(wad_file, map_name);

        for linedef in &linedefs {
            let start_vertex = &vertexes[linedef.start_vertex as usize];
            let end_vertex = &vertexes[linedef.end_vertex as usize];

            min_x = std::cmp::min(min_x, start_vertex.x);
            min_x = std::cmp::min(min_x, end_vertex.x);
            max_x = std::cmp::max(max_x, start_vertex.x);
            max_x = std::cmp::max(max_x, end_vertex.x);

            min_y = std::cmp::min(min_y, start_vertex.y);
            min_y = std::cmp::min(min_y, end_vertex.y);
            max_y = std::cmp::max(max_y, start_vertex.y);
            max_y = std::cmp::max(max_y, end_vertex.y);
        }

        Map {
            vertexes: vertexes,
            linedefs: linedefs,
            segs: segs,
            subsectors: subsectors,
            nodes: nodes,
            top_left: Vertex { x: min_x, y: min_y },
            bottom_right: Vertex { x: max_x, y: max_y },
        }
    }
}
