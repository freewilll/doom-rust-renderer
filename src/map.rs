use crate::linedefs::{load_linedefs, Linedef};
use crate::vertexes::{load_vertexes, Vertex};
use crate::wad::WadFile;

#[allow(dead_code)]
pub struct Map {
    vertexes: Vec<Vertex>,
    linedefs: Vec<Linedef>,
}

impl Map {
    // Load map
    pub fn new(wad_file: &WadFile, map_name: &str) -> Map {
        Map {
            vertexes: load_vertexes(wad_file, map_name),
            linedefs: load_linedefs(wad_file, map_name),
        }
    }
}
