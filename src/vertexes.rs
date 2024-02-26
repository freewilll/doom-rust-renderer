use crate::wad::{MapLumpName, WadFile};
use std::ops::{Add, Sub};
use std::rc::Rc;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
}

impl Vertex {
    pub fn new(x: i16, y: i16) -> Vertex {
        Vertex { x: x, y: y }
    }

    pub fn rotate(&self, angle: f32) -> Vertex {
        Vertex {
            x: (self.x as f32 * angle.cos()) as i16,
            y: (self.y as f32 * angle.sin()) as i16,
        }
    }
}

impl<'a, 'b> Add<&'b Vertex> for &'a Vertex {
    type Output = Vertex;

    fn add(self, other: &'b Vertex) -> Vertex {
        Vertex {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<'a, 'b> Sub<&'b Vertex> for &'a Vertex {
    type Output = Vertex;

    fn sub(self, other: &'b Vertex) -> Vertex {
        Vertex {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

pub fn load_vertexes(wad_file: &WadFile, map_name: &str) -> Vec<Rc<Vertex>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Vertexes);
    let count = dir_entry.size as usize / 4; // A vertex is 4 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 4;
        let vertex = Vertex {
            x: wad_file.read_i16(offset),
            y: wad_file.read_i16(offset + 2),
        };
        results.push(Rc::new(vertex));
    }

    results
}
