use sdl2::rect::Point;

use crate::game::SCREEN_WIDTH;
use crate::geometry::Line;
use crate::vertexes::Vertex;

use super::clipped_line::ClippedLine;
use super::constants::{
    ASPECT_RATIO_CORRECTION, CAMERA_FOCUS_X, CAMERA_FOCUS_Y, GAME_CAMERA_FOCUS_X,
};
use super::sdl_line::SdlLine;

pub fn clip_to_viewport(line: &Line) -> Option<ClippedLine> {
    // Clip a line in player coordinates to the viewport

    // 45 degree viewport lines:
    let left = Line::new(&Vertex::new(0.0, 0.0), &Vertex::new(1.0, 1.0));
    let right = Line::new(&Vertex::new(0.0, 0.0), &Vertex::new(1.0, -1.0));

    // Find where the start & ends of the line fall with respect to the clipping
    // lines.
    let start_outside_left = line.start.is_left_of_line(&left);
    let end_outside_left = line.end.is_left_of_line(&left);

    let start_outside_right = !line.start.is_left_of_line(&right);
    let end_outside_right = !line.end.is_left_of_line(&right);

    // Determine if the start & end of the line is inside the viewport
    let start_in_viewport = line.start.x > 0.0 && !start_outside_left && !start_outside_right;
    let end_in_viewport = line.end.x > 0.0 && !end_outside_left && !end_outside_right;

    // If the line is entirely in the viewport, no clipping is needed
    if start_in_viewport && end_in_viewport {
        return Some(ClippedLine {
            line: line.clone(),
            start_offset: 0.0,
        });
    }

    // Determine intersections with the viewport
    let left_intersection = line.intersection(&left);
    let right_intersection = line.intersection(&right);

    // Determine if the wall intersects the viewport in front of us
    let left_intersected = if let Ok(left_intersection) = left_intersection.clone() {
        left_intersection.x >= 0.0
    } else {
        false
    };

    let right_intersected = if let Ok(right_intersection) = right_intersection.clone() {
        right_intersection.x >= 0.0
    } else {
        false
    };

    // If the line is entirely outside of the viewport, there are two cases:
    // - The wall is in front of us and has intersections in the viewport: it's visible
    // - Otherwise: it's not in view
    if !start_in_viewport && !end_in_viewport && !left_intersected && !right_intersected {
        return None;
    }

    // If neither start nor end of the line is in the viewport and there is one intersection, then
    // the line doesn't cross the viewport.
    if !start_in_viewport && !end_in_viewport && (left_intersected != right_intersected) {
        return None;
    }

    // Eliminate lines that intersect the viewport but are outside it
    if (right_intersected && start_outside_right && end_outside_right)
        || (left_intersected && start_outside_left && end_outside_left)
    {
        return None;
    }

    // Clipping is needed
    let mut start_offset: f32 = 0.0; // The amount of clipping happened on the left

    let mut start = line.start.clone();
    let mut end = line.end.clone();

    if left_intersected {
        // Clip start outside left viewport
        if start_outside_left {
            let new_start = left_intersection.clone().unwrap();
            start_offset = new_start.distance_to(&start);
            start = new_start;
        }

        // Clip end outside left viewport
        if end_outside_left {
            end = left_intersection.clone().unwrap();
        }
    }

    if right_intersected {
        // Clip start outside right viewport
        if start_outside_right {
            start = right_intersection.clone().unwrap();
        }

        // Clip end outside right viewport
        if end_outside_right {
            end = right_intersection.clone().unwrap();
        }
    }

    let clipped_line = ClippedLine {
        line: Line::new(&start, &end),
        start_offset,
    };

    Some(clipped_line)
}

// Transform a vertex in doom x-y coordinates to viewport coordinates.
// Player:
//    x
//    |
// <- y
//
// Viewport:
// \  x  /
//  \ ^ /
//   \|/
//     -----> y
//
// https://en.wikipedia.org/wiki/3D_projection#Weak_perspective_projection
fn perspective_transform(v: &Vertex, y: f32) -> Vertex {
    let x = v.y;
    let z = v.x;

    Vertex::new(GAME_CAMERA_FOCUS_X * x / z, GAME_CAMERA_FOCUS_X * y / z)
}

// Make the slanted non-vertical line for a sidedef.
pub fn make_sidedef_non_vertical_line(line: &Line, height: f32) -> SdlLine {
    let mut transformed_start = perspective_transform(&line.start, height);
    let mut transformed_end = perspective_transform(&line.end, height);

    // Convert the in-game coordinates that are broad into the more narrow
    // screen x coordinates
    transformed_start.x *= ASPECT_RATIO_CORRECTION;
    transformed_end.x *= ASPECT_RATIO_CORRECTION;

    let mut screen_start = Point::new(
        (CAMERA_FOCUS_X - transformed_start.x) as i32,
        (CAMERA_FOCUS_Y - transformed_start.y) as i32,
    );

    let mut screen_end = Point::new(
        (CAMERA_FOCUS_X - transformed_end.x) as i32,
        (CAMERA_FOCUS_Y - transformed_end.y) as i32,
    );

    screen_start.x = screen_start.x.min(SCREEN_WIDTH as i32 - 1);
    screen_end.x = screen_end.x.min(SCREEN_WIDTH as i32 - 1);

    SdlLine::new(&screen_start, &screen_end)
}
