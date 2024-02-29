use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::rc::Rc;

use crate::game::{Game, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::geometry::Line;
use crate::nodes::{Node, NodeChild};
use crate::segs::Seg;
use crate::subsectors::SubSector;
use crate::vertexes::Vertex;

const PLAYER_HEIGHT: f32 = 56.0;

// A couple of test colors used for easy visual development
#[allow(dead_code)]
const COLORS: &'static [Color] = &[
    Color::RGB(0, 0, 255),   // Blue
    Color::RGB(0, 255, 0),   // Green
    Color::RGB(0, 255, 255), // Aqua
    Color::RGB(255, 0, 0),   // Red
    Color::RGB(255, 0, 255), // Purple
    Color::RGB(255, 255, 0), // Yellow
];

// Length of the viewport from the player looking forward along the x axis
const CAMERA_FOCUS: f32 = SCREEN_WIDTH as f32 / 2.0 as f32;

// Transform a vertex in doom x-y coordinates to viewport coordinates.
// PLayer:
//    x
//    |
// <- y
//
// Viewport:
// \  x  /
//  \ ^ /
//   \|/
//     -----> y
fn perspective_transform(v: &Vertex, y: f32) -> Vertex {
    let x = v.y;
    let z = v.x;

    Vertex::new(CAMERA_FOCUS * x / z, CAMERA_FOCUS * y / z)
}

#[allow(dead_code)]
fn draw_seg_on_2d_map(game: &mut Game, seg: &Seg) {
    // Draw the segment coordinates on the 2D map

    let map_start = game.transform_vertex_to_point_for_map(&*seg.start_vertex);
    let map_end = game.transform_vertex_to_point_for_map(&*seg.end_vertex);
    game.canvas.draw_line(map_start, map_end).unwrap();
}

fn clip_to_viewport(line: &Line) -> Option<Line> {
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
        return Some(line.clone());
    }

    // If the line is entirely outside of the viewport, don't render it
    // FIXME: this makes a wall the player is facing disappear if they get too close
    // and the start/end both fall outside of the viewport.
    if !start_in_viewport && !end_in_viewport {
        return None;
    }

    // Clipping is needed
    let mut start = line.start.clone();
    let mut end = line.end.clone();

    // Clip start outside left viewport
    if start_outside_left {
        if let Ok(left_intersection) = line.intersection(&left) {
            if left_intersection.x >= 0.0 {
                start = left_intersection;
            }
        }
    }

    // Clip end outside left viewport
    if end_outside_left {
        if let Ok(left_intersection) = line.intersection(&left) {
            if left_intersection.x >= 0.0 {
                end = left_intersection;
            }
        }
    }

    // Clip start outside right viewport
    if start_outside_right {
        if let Ok(right_intersection) = line.intersection(&right) {
            if right_intersection.x >= 0.0 {
                start = right_intersection;
            }
        }
    }

    // Clip end outside right viewport
    if end_outside_right {
        if let Ok(right_intersection) = line.intersection(&right) {
            if right_intersection.x >= 0.0 {
                end = right_intersection;
            }
        }
    }

    Some(Line::new(&start, &end))
}

// Draw a wall's top or bottom line
fn draw_wall_floor_or_ceiling(game: &mut Game, line: &Line, height: f32) -> Option<(Point, Point)> {
    let transformed_start = perspective_transform(&line.start, height);
    let transformed_end = perspective_transform(&line.end, height);

    let screen_start = Point::new(
        ((-&transformed_start.x + CAMERA_FOCUS) as i32).into(),
        ((SCREEN_HEIGHT as f32 / 2.0 - &transformed_start.y - 1.0) as i32).into(),
    );

    let screen_end = Point::new(
        ((-&transformed_end.x + CAMERA_FOCUS) as i32).into(),
        ((SCREEN_HEIGHT as f32 / 2.0 - &transformed_end.y - 1.0) as i32).into(),
    );

    if screen_start.x > screen_end.x {
        return None;
    }

    game.canvas.draw_line(screen_start, screen_end).unwrap();

    Some((screen_start, screen_end))
}

// Draw a seg
fn render_seg(game: &mut Game, seg: &Seg) {
    let linedef = &seg.linedef;

    let sidedef = if seg.direction {
        &linedef.back_sidedef
    } else {
        &linedef.front_sidedef
    };

    // No sector, no drawing
    let sector = match sidedef {
        Some(s) => &s.sector,
        None => {
            return;
        }
    };

    // Update player height if this is the first rendered seg
    if !game.player_height_updated {
        game.player_floor_height = sector.floor_height as f32;
        game.player_height_updated = true;
    }

    let floor_height = sector.floor_height as f32;
    let ceiling_height = sector.ceiling_height as f32;

    // Move the seg's vertex to the player
    let moved_start = &*seg.start_vertex - &game.player.position;
    let moved_end = &*seg.end_vertex - &game.player.position;

    // Rotate the seg by the player angle
    let start = moved_start.rotate(-game.player.angle);
    let end = moved_end.rotate(-game.player.angle);

    let line = Line::new(&start, &end);

    if let Some(clipped_line) = clip_to_viewport(&line) {
        if clipped_line.start.x < 0.01 {
            panic!(
                "Clipped line x <= 0.0 {:?} player: {:?}",
                &clipped_line.start, &game.player.position
            );
        }

        // Set line color
        game.canvas
            .set_draw_color(COLORS[seg.id as usize % COLORS.len()]);

        let player_height = &game.player_floor_height + PLAYER_HEIGHT;

        // Draw the floor & ceiling lines
        let floor = draw_wall_floor_or_ceiling(game, &clipped_line, ceiling_height - player_height);
        let ceiling = draw_wall_floor_or_ceiling(game, &clipped_line, floor_height - player_height);

        // Draw the left and right vertial lines
        if floor != None && ceiling != None {
            let (floor_screen_start, floor_screen_end) = floor.unwrap();
            let (ceiling_screen_start, ceiling_screen_end) = ceiling.unwrap();
            game.canvas
                .draw_line(floor_screen_start, ceiling_screen_start)
                .unwrap();
            game.canvas
                .draw_line(floor_screen_end, ceiling_screen_end)
                .unwrap();
        }
    }
}

// Render all segs in a subsector
fn render_subsector(game: &mut Game, subsector: &SubSector) {
    for seg in &subsector.segs {
        render_seg(game, &seg);
    }
}

// Recurse through the BSP tree, drawing the subsector leaves
// The BSP algorithm guarantees that the subsectors are visited back to front.
fn render_node(game: &mut Game, node: &Rc<Node>) {
    let v1 = Vertex::new(node.x, node.y);
    let v2 = &v1 + &Vertex::new(node.dx, node.dy);

    let is_left = game.player.position.is_left_of_line(&Line::new(&v1, &v2));

    let (front_child, back_child) = if is_left {
        (&node.left_child, &node.right_child)
    } else {
        (&node.right_child, &node.left_child)
    };

    match front_child {
        NodeChild::Node(node) => {
            render_node(game, &node);
        }
        NodeChild::SubSector(subsector) => {
            render_subsector(game, &subsector);
        }
    }

    // TODO: Use the bounding box and only recurse into the back of the split
    // if the player viewintersecrs with it.
    match back_child {
        NodeChild::Node(node) => {
            render_node(game, &node);
        }
        NodeChild::SubSector(subsector) => {
            render_subsector(game, &subsector);
        }
    }
}

pub fn render_map(game: &mut Game) {
    game.player_height_updated = false;
    let root_node = Rc::clone(&game.map.root_node);
    render_node(game, &root_node);
}
