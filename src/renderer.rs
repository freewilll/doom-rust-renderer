use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::rc::Rc;

use crate::game::{Game, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::geometry::Line;
use crate::linedefs::Flags;
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

#[derive(Debug, PartialEq)]
struct SdlLine {
    start: Point,
    end: Point,
}

impl SdlLine {
    fn new(start: &Point, end: &Point) -> SdlLine {
        SdlLine {
            start: start.clone(),
            end: end.clone(),
        }
    }
}
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

    // Determine intersections with the viewport
    let left_intersection = line.intersection(&left);
    let right_intersection = line.intersection(&right);

    // Determine if the wall intersects the viewport in front of us
    let left_intersected = if let Ok(left_intersection) = left_intersection.clone() {
        if left_intersection.x >= 0.0 {
            true
        } else {
            false
        }
    } else {
        false
    };

    let right_intersected = if let Ok(right_intersection) = right_intersection.clone() {
        if right_intersection.x >= 0.0 {
            true
        } else {
            false
        }
    } else {
        false
    };

    // If the line is entirely outside of the viewport, there are two cases:
    // - The wall is in front of us and has intersections in the viewport: it's visible
    // - Otherwise: it's not in view
    if !start_in_viewport && !end_in_viewport && !left_intersected && !right_intersected {
        return None;
    }

    // Clipping is needed
    let mut start = line.start.clone();
    let mut end = line.end.clone();

    if left_intersected {
        // Clip start outside left viewport
        if start_outside_left {
            start = left_intersection.clone().unwrap();
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

    Some(Line::new(&start, &end))
}

// Draw a wall's non-vertical line
fn draw_wall_floor_or_ceiling_line(game: &mut Game, line: &Line, height: f32) -> Option<SdlLine> {
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

    Some(SdlLine::new(&screen_start, &screen_end))
}

// Draw a part of a wall. This can be either of the lower, middle and upper textures
fn draw_wall(game: &mut Game, bottom: &SdlLine, top: &SdlLine) {
    // Sanity check the wall is vertical
    if bottom.start.x != top.start.x {
        panic!(
            "Wall start not vertical: {} vs {}",
            &bottom.start.x, &top.start.x
        );
    }

    if bottom.end.x != top.end.x {
        panic!("Wall end not vertical: {} vs {}", &bottom.end.x, &top.end.x);
    }

    // Loop from the left x to the right x, calculating the y screen coordinates
    // for the bottom and top.
    let bottom_delta = (bottom.start.y as f32 - bottom.end.y as f32)
        / (bottom.start.x as f32 - bottom.end.x as f32);
    let top_delta =
        (top.start.y as f32 - top.end.y as f32) / (top.start.x as f32 - top.end.x as f32);

    for x in bottom.start.x as i16..bottom.end.x as i16 + 1 {
        let bottom_y = bottom.start.y as f32 + (x as f32 - bottom.start.x as f32) * bottom_delta;
        let top_y = top.start.y as f32 + (x as f32 - top.start.x as f32) * top_delta;

        game.canvas
            .draw_line(
                Point::new(x as i32, bottom_y as i32),
                Point::new(x as i32, top_y as i32),
            )
            .unwrap();
    }
}

// Draw a seg
fn render_seg(game: &mut Game, seg: &Seg) {
    // Get the linedef
    let linedef = &seg.linedef;

    // Get the sidedef(s)
    let (opt_front_sidedef, opt_back_sidedef) = if seg.direction {
        (&linedef.back_sidedef, &linedef.front_sidedef)
    } else {
        (&linedef.front_sidedef, &linedef.back_sidedef)
    };

    // Get the front sector (the one we're facing)
    let front_sector = match opt_front_sidedef {
        Some(s) => &s.sector,
        None => {
            // If there is no sidedef, then there is no wall
            return;
        }
    };

    // Get the floor and ceiling height from the front sector
    let floor_height = front_sector.floor_height as f32;
    let ceiling_height = front_sector.ceiling_height as f32;

    // For portals, get the bottom and top heights by looking at the back
    // sector.
    let (opt_portal_bottom_height, opt_portal_top_height) = match opt_back_sidedef {
        Some(back_sidedef) => {
            let back_sector = &back_sidedef.sector;

            let opt_portal_bottom_height = if back_sector.floor_height > front_sector.floor_height {
                Some(back_sector.floor_height as f32)
            } else {
                None
            };

            let opt_portal_top_height = if back_sector.ceiling_height < front_sector.ceiling_height
            {
                Some(back_sector.ceiling_height as f32)
            } else {
                None
            };

            (opt_portal_bottom_height, opt_portal_top_height)
        }
        None => (None, None),
    };

    let is_solid_wall = opt_back_sidedef.is_none();
    let is_two_sided = linedef.flags & Flags::TWOSIDED != 0;

    // Don't render walls that are entirely transparent
    if is_two_sided && is_solid_wall {
        return;
    }

    // Transform the seg so that the player position and angle is transformed
    // away.

    let moved_start = &*seg.start_vertex - &game.player.position;
    let moved_end = &*seg.end_vertex - &game.player.position;

    let start = moved_start.rotate(-game.player.angle);
    let end = moved_end.rotate(-game.player.angle);

    // The coordinates of line are like this:
    // y
    // ^
    // |
    //  -> x
    let line = Line::new(&start, &end);

    let clipped_line = match clip_to_viewport(&line) {
        Some(clipped_line) => clipped_line,
        None => {
            return;
        }
    };

    if clipped_line.start.x < -0.01 {
        panic!(
            "Clipped line x < -0.01: {:?} player: {:?}",
            &clipped_line.start.x, &game.player.position
        );
    }

    // Set line color
    game.canvas
        .set_draw_color(COLORS[seg.id as usize % COLORS.len()]);

    // Draw the non-vertial lines for all parts of the wall
    let player_height = &game.player_floor_height + PLAYER_HEIGHT;

    // Draw the floor & ceiling lines
    let floor = draw_wall_floor_or_ceiling_line(game, &clipped_line, floor_height - player_height);
    let ceiling =
        draw_wall_floor_or_ceiling_line(game, &clipped_line, ceiling_height - player_height);

    let portal_bottom = if let Some(portal_bottom_height) = opt_portal_bottom_height {
        Some(draw_wall_floor_or_ceiling_line(
            game,
            &clipped_line,
            portal_bottom_height - player_height,
        ))
    } else {
        None
    };

    let portal_top = if let Some(portal_top_height) = opt_portal_top_height {
        Some(draw_wall_floor_or_ceiling_line(
            game,
            &clipped_line,
            portal_top_height - player_height,
        ))
    } else {
        None
    };

    // We now have all the non-vertial lines, draw the walls in between them.
    if let Some(floor) = floor {
        if let Some(ceiling) = ceiling {
            if !is_two_sided {
                // Draw a solid wall's middle texture, floor to ceiling

                draw_wall(game, &floor, &ceiling);
            } else {
                // Draw a portal's lower and upper textures (if present)

                if let Some(pb) = portal_bottom {
                    if let Some(pb) = pb {
                        draw_wall(game, &floor, &pb);
                    }
                }

                if let Some(pt) = portal_top {
                    if let Some(pt) = pt {
                        draw_wall(game, &pt, &ceiling);
                    }
                }
            }
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
// The BSP algorithm guarantees that the subsectors are visited either back to front
// or in reverse. Here, we go from back to front and use the painter's algorithm.
fn render_node(game: &mut Game, node: &Rc<Node>) {
    let v1 = Vertex::new(node.x, node.y);
    let v2 = &v1 + &Vertex::new(node.dx, node.dy);

    let is_left = game.player.position.is_left_of_line(&Line::new(&v1, &v2));

    let (front_child, back_child) = if is_left {
        (&node.left_child, &node.right_child)
    } else {
        (&node.right_child, &node.left_child)
    };

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

    match front_child {
        NodeChild::Node(node) => {
            render_node(game, &node);
        }
        NodeChild::SubSector(subsector) => {
            render_subsector(game, &subsector);
        }
    }
}

pub fn render_map(game: &mut Game) {
    let root_node = Rc::clone(&game.map.root_node);
    render_node(game, &root_node);
}
