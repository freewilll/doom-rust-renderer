use sdl2::rect::Point;

#[derive(Debug, PartialEq, Clone)]
pub struct SdlLine {
    pub start: Point,
    pub end: Point,
}

impl SdlLine {
    pub fn new(start: &Point, end: &Point) -> SdlLine {
        SdlLine {
            start: start.clone(),
            end: end.clone(),
        }
    }
}
