use crate::vertexes::Vertex;

#[derive(Debug)]
#[allow(dead_code)]
pub struct BoundingBox {
    pub top: i16,
    pub bottom: i16,
    pub left: i16,
    pub right: i16,
}

impl BoundingBox {
    // Create a new bounding box suitable to be extended by calling extend
    pub fn extendable_new() -> BoundingBox {
        BoundingBox {
            left: i16::MAX,
            right: i16::MIN,
            top: i16::MAX,
            bottom: i16::MIN,
        }
    }

    // Extend bounding box to include a vertex
    pub fn extend(&mut self, v: &Vertex) {
        self.left = std::cmp::min(self.left, v.x);
        self.right = std::cmp::max(self.right, v.x);
        self.top = std::cmp::min(self.top, v.y);
        self.bottom = std::cmp::max(self.bottom, v.y);
    }
}
