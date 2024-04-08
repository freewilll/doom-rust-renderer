use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};
use std::rc::Rc;

use crate::geometry::Line;
use crate::wad::{MapLumpName, WadFile};

#[derive(Clone, Deserialize, Serialize)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
}

impl Vertex {
    pub fn new(x: f32, y: f32) -> Vertex {
        Vertex { x, y }
    }

    pub fn rotate(&self, angle: f32) -> Vertex {
        Vertex {
            x: self.x * angle.cos() - self.y * angle.sin(),
            y: self.y * angle.cos() + self.x * angle.sin(),
        }
    }

    pub fn cross_product(&self, other: &Vertex) -> f32 {
        self.x * other.y - self.y * other.x
    }

    // Are we left or of the line ?
    pub fn is_left_of_line(&self, line: &Line) -> bool {
        (self - &line.start).cross_product(&(&line.end - &line.start)) <= 0.0
    }

    pub fn distance_to(&self, other: &Vertex) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl fmt::Debug for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
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
            x: wad_file.read_f32_from_i16(offset),
            y: wad_file.read_f32_from_i16(offset + 2),
        };
        results.push(Rc::new(vertex));
    }

    results
}
