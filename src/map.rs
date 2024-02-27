use crate::geometry::BoundingBox;
use crate::linedefs::{load_linedefs, Linedef};
use crate::nodes::{load_nodes, Node};
use crate::sectors::{load_sectors, Sector};
use crate::segs::{load_segs, Seg};
use crate::sidedefs::{load_sidedefs, Sidedef};
use crate::subsectors::{load_subsectors, SubSector};
use crate::things::{load_things, Thing};
use crate::vertexes::{load_vertexes, Vertex};
use crate::wad::WadFile;
use std::rc::Rc;

#[allow(dead_code)]
pub struct Map {
    pub things: Vec<Rc<Thing>>,         // Monsters, weapons, keys, etc
    pub vertexes: Vec<Rc<Vertex>>,      // Vertexes that make up the lines
    pub linedefs: Vec<Rc<Linedef>>,     // Lines
    pub sidedefs: Vec<Rc<Sidedef>>,     // What's on the side of a line
    pub segs: Vec<Rc<Seg>>,             // Lines, split by the BSP builder
    pub subsectors: Vec<Rc<SubSector>>, // Sectors, split by the BSP builder
    pub nodes: Vec<Rc<Node>>,           // BSP tree
    pub sectors: Vec<Rc<Sector>>,       // Closed polygons made up of linedefs
    pub root_node: Rc<Node>,            // Root node of the BSP tree
    pub bounding_box: BoundingBox,      // Bounding box for the whole map
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
            things: things,
            vertexes: vertexes,
            linedefs: linedefs,
            sidedefs: sidedefs,
            segs: segs,
            subsectors: subsectors,
            nodes: nodes,
            sectors: sectors,
            root_node: root_node,
            bounding_box: bounding_box,
        }
    }
}
