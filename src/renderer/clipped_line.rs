use crate::geometry::Line;

#[derive(Debug, Clone)]
pub struct ClippedLine {
    pub line: Line,
    pub start_offset: f32, // The amount the line was clipped by at the start/left end
}
