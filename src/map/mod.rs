mod linedefs;
mod nodes;
mod sectors;
mod segs;
mod sidedefs;
mod subsectors;
mod things;
mod vertexes;

use std::cell::RefCell;
use std::rc::Rc;

use crate::map::{
    linedefs::load_linedefs, nodes::load_nodes, sectors::load_sectors, segs::load_segs,
    sidedefs::load_sidedefs, subsectors::load_subsectors, things::load_things,
    vertexes::load_vertexes,
};

pub use crate::geometry::BoundingBox;
pub use crate::map::{
    linedefs::{Flags, Linedef},
    nodes::{Node, NodeChild},
    sectors::Sector,
    segs::Seg,
    sidedefs::Sidedef,
    subsectors::SubSector,
    things::{get_thing_by_type, Thing, ThingTypes},
    vertexes::Vertex,
};
pub use crate::wad::WadFile;

#[allow(dead_code)]
pub struct Map {
    pub things: Vec<Rc<Thing>>,            // Monsters, weapons, keys, etc
    pub vertexes: Vec<Rc<Vertex>>,         // Vertexes that make up the lines
    pub linedefs: Vec<Rc<Linedef>>,        // Lines
    pub sidedefs: Vec<Rc<Sidedef>>,        // What's on the side of a line
    pub segs: Vec<Rc<Seg>>,                // Lines, split by the BSP builder
    pub subsectors: Vec<Rc<SubSector>>,    // Sectors, split by the BSP builder
    pub nodes: Vec<Rc<Node>>,              // BSP tree
    pub sectors: Vec<Rc<RefCell<Sector>>>, // Closed polygons made up of linedefs, mutatable with RefCell
    pub root_node: Rc<Node>,               // Root node of the BSP tree
    pub bounding_box: BoundingBox,         // Bounding box for the whole map
}

impl Map {
    // Load map
    pub fn new(wad_file: &WadFile, map_name: &str) -> Map {
        let things = load_things(wad_file, map_name);
        let vertexes = load_vertexes(wad_file, map_name);
        let sectors = load_sectors(wad_file, map_name);
        let sidedefs = load_sidedefs(wad_file, &sectors, map_name);
        let linedefs = load_linedefs(wad_file, &vertexes, &sidedefs, map_name);
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
            things,
            vertexes,
            linedefs,
            sidedefs,
            segs,
            subsectors,
            nodes,
            sectors,
            root_node,
            bounding_box,
        }
    }
}
