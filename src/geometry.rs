use crate::vertexes::Vertex;

#[derive(Debug)]
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

    // Is a vertex in the bounding box?
    #[allow(dead_code)]
    pub fn contains(&self, vertex: &Vertex) -> bool {
        self.left <= vertex.x
            && self.right >= vertex.x
            && self.top <= vertex.y
            && self.bottom <= vertex.x
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub start: Vertex,
    pub end: Vertex,
}

impl Line {
    #[allow(dead_code)]
    pub fn new(start: &Vertex, end: &Vertex) -> Line {
        Line {
            start: start.clone(),
            end: end.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn intersection(&self, other: &Line) -> Result<Vertex, String> {
        // Returns the point of intersection or Err if they are parallel
        // http://en.wikipedia.org/wiki/Line-line_intersection

        let x1 = self.start.x;
        let y1 = self.start.y;
        let x2 = self.end.x;
        let y2 = self.end.y;

        let x3 = other.start.x;
        let y3 = other.start.y;
        let x4 = other.end.x;
        let y4 = other.end.y;

        let quot = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

        if quot.abs() < 0.001 {
            return Err("Lines are parallel".into());
        }

        let invquot = 1.0 / quot;

        let px = invquot * ((x1 * y2 - y1 * x2) * (x3 - x4) - (x1 - x2) * (x3 * y4 - y3 * x4));
        let py = invquot * ((x1 * y2 - y1 * x2) * (y3 - y4) - (y1 - y2) * (x3 * y4 - y3 * x4));

        Ok(Vertex::new(px, py))
    }

    pub fn length(&self) -> f32 {
        ((self.start.x - self.end.x).powi(2) + (self.start.y - self.end.y).powi(2)).sqrt()
    }

    // Is either the start or end point of our line on the left of another line?
    pub fn is_left_of_line(&self, other: &Line) -> bool {
        self.start.is_left_of_line(other) || self.end.is_left_of_line(other)
    }
}
