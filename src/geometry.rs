use crate::vertexes::Vertex;

#[derive(Debug)]
#[allow(dead_code)]
pub struct BoundingBox {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl BoundingBox {
    // Create a new bounding box suitable to be extended by calling extend
    pub fn extendable_new() -> BoundingBox {
        BoundingBox {
            left: f32::MAX,
            right: f32::MIN,
            top: f32::MAX,
            bottom: f32::MIN,
        }
    }

    // Extend bounding box to include a vertex
    pub fn extend(&mut self, v: &Vertex) {
        self.left = self.left.min(v.x);
        self.right = self.right.max(v.x);
        self.top = self.top.min(v.y);
        self.bottom = self.bottom.max(v.y);
    }
}
