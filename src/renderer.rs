use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::cmp::{max, min};
use std::rc::Rc;

use crate::game::{Game, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::geometry::Line;
use crate::linedefs::Flags;
use crate::nodes::{Node, NodeChild};
use crate::segs::Seg;
use crate::subsectors::SubSector;
use crate::vertexes::Vertex;

const PLAYER_HEIGHT: f32 = 56.0;

// Length of the viewport from the player looking forward along the x axis
const CAMERA_FOCUS: f32 = SCREEN_WIDTH as f32 / 2.0 as f32;

// A couple of test colors used for easy visual development
// From https://www.rapidtables.com/web/color/RGB_Color.html
#[allow(dead_code)]
const WALL_COLORS: &'static [Color] = &[
    Color::RGB(128, 0, 0),     // maroon
    Color::RGB(139, 0, 0),     // dark red
    Color::RGB(165, 42, 42),   // brown
    Color::RGB(178, 34, 34),   // firebrick
    Color::RGB(220, 20, 60),   // crimson
    Color::RGB(255, 0, 0),     // red
    Color::RGB(255, 99, 71),   // tomato
    Color::RGB(255, 127, 80),  // coral
    Color::RGB(205, 92, 92),   // indian red
    Color::RGB(240, 128, 128), // light coral
    Color::RGB(233, 150, 122), // dark salmon
    Color::RGB(250, 128, 114), // salmon
    Color::RGB(255, 160, 122), // light salmon
    Color::RGB(255, 69, 0),    // orange red
    Color::RGB(255, 140, 0),   // dark orange
    Color::RGB(255, 165, 0),   // orange
    Color::RGB(255, 215, 0),   // gold
];

#[allow(dead_code)]
const VISPLANE_COLORS: &'static [Color] = &[
    Color::RGB(0, 128, 128),   //teal
    Color::RGB(0, 139, 139),   //dark cyan
    Color::RGB(0, 255, 255),   //aqua
    Color::RGB(0, 255, 255),   //cyan
    Color::RGB(224, 255, 255), //light cyan
    Color::RGB(0, 206, 209),   //dark turquoise
    Color::RGB(64, 224, 208),  //turquoise
    Color::RGB(72, 209, 204),  //medium turquoise
    Color::RGB(175, 238, 238), //pale turquoise
    Color::RGB(127, 255, 212), //aqua marine
    Color::RGB(176, 224, 230), //powder blue
    Color::RGB(95, 158, 160),  //cadet blue
    Color::RGB(70, 130, 180),  //steel blue
    Color::RGB(100, 149, 237), //corn flower blue
    Color::RGB(0, 191, 255),   //deep sky blue
    Color::RGB(30, 144, 255),  //dodger blue
];

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Visplane {
    // Describes a floor or ceiling area bounded by vertical left and right lines.
    floor_texture_hash: i16,
    left: i16,                            // Minimum x coordinate
    right: i16,                           // Maximum x coordinate
    top: [i16; SCREEN_WIDTH as usize],    // Top line
    bottom: [i16; SCREEN_WIDTH as usize], // Bottom line
}

impl Visplane {
    fn new(floor_texture_hash: i16) -> Visplane {
        Visplane {
            floor_texture_hash: floor_texture_hash,
            left: -1,
            right: -1,
            top: [0; SCREEN_WIDTH as usize],
            bottom: [0; SCREEN_WIDTH as usize],
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Context {
    hor_ocl: [bool; SCREEN_WIDTH as usize], // Horizontal occlusions
    floor_ver_ocl: [i16; SCREEN_WIDTH as usize], // Vertical occlusions for the floor
    ceiling_ver_ocl: [i16; SCREEN_WIDTH as usize], // Vertical occlusions for the ceiling
    visplanes: Vec<Visplane>,
}

impl Context {
    fn occlude_vertical_line(&mut self, x: i16) {
        self.hor_ocl[x as usize] = true;
        self.floor_ver_ocl[x as usize] = SCREEN_HEIGHT as i16 / 2;
        self.ceiling_ver_ocl[x as usize] = SCREEN_HEIGHT as i16 / 2;
    }
}

#[derive(Debug, PartialEq, Clone)]
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

// Make the slanted non-vertical line for a sidedef & check the orientation.
fn make_sidedef_non_vertical_line(line: &Line, height: f32) -> SdlLine {
    let transformed_start = perspective_transform(&line.start, height);
    let transformed_end = perspective_transform(&line.end, height);

    let mut screen_start = Point::new(
        ((CAMERA_FOCUS - &transformed_start.x) as i32).into(),
        ((SCREEN_HEIGHT as f32 / 2.0 - &transformed_start.y - 1.0) as i32).into(),
    );

    let mut screen_end = Point::new(
        ((-&transformed_end.x + CAMERA_FOCUS) as i32).into(),
        ((SCREEN_HEIGHT as f32 / 2.0 - &transformed_end.y - 1.0) as i32).into(),
    );

    if screen_start.x >= SCREEN_WIDTH as i32 {
        screen_start.x = SCREEN_WIDTH as i32 - 1
    }

    if screen_end.x >= SCREEN_WIDTH as i32 {
        screen_end.x = SCREEN_WIDTH as i32 - 1
    }

    SdlLine::new(&screen_start, &screen_end)
}

fn draw_visplane(game: &mut Game, visplane: &Visplane) {
    const DEBUG_DRAW_OUTLINE: bool = false;

    let solid_color = VISPLANE_COLORS[visplane.floor_texture_hash as usize % VISPLANE_COLORS.len()];
    let outline_color = Color::RGB(255, 255, 255);

    for x in visplane.left..visplane.right + 1 {
        let top = visplane.top[x as usize];
        let bottom = visplane.bottom[x as usize];

        let top_point = Point::new(x as i32, top as i32);
        let bottom_point = Point::new(x as i32, bottom as i32);

        game.canvas.set_draw_color(solid_color);
        game.canvas.draw_line(top_point, bottom_point).unwrap();

        if DEBUG_DRAW_OUTLINE {
            game.canvas.set_draw_color(outline_color);
            game.canvas
                .draw_points([top_point, bottom_point].as_slice())
                .unwrap();
        }
    }

    if DEBUG_DRAW_OUTLINE {
        game.canvas.set_draw_color(outline_color);

        let left = visplane.left as i32;
        let right = visplane.right as i32;

        game.canvas
            .draw_line(
                Point::new(left, visplane.bottom[left as usize] as i32),
                Point::new(left, visplane.top[left as usize] as i32),
            )
            .unwrap();

        game.canvas
            .draw_line(
                Point::new(right, visplane.bottom[right as usize] as i32),
                Point::new(right, visplane.top[right as usize] as i32),
            )
            .unwrap();
    }
}

fn draw_visplanes(game: &mut Game, context: &Context) {
    for visplane in &context.visplanes {
        draw_visplane(game, &visplane);
    }
}

// Keep track of the visplane state while processing a sidedef
struct SidedefVisPlanes {
    floor_texture_hash: i16,
    ceiling_texture_hash: i16,
    bottom_visplane: Visplane,
    top_visplane: Visplane,
    bottom_visplane_used: bool,
    top_visplane_used: bool,
}

impl SidedefVisPlanes {
    fn new(floor_texture_hash: i16, ceiling_texture_hash: i16) -> SidedefVisPlanes {
        SidedefVisPlanes {
            floor_texture_hash: floor_texture_hash,
            ceiling_texture_hash: ceiling_texture_hash,
            bottom_visplane: Visplane::new(floor_texture_hash),
            bottom_visplane_used: false,
            top_visplane: Visplane::new(ceiling_texture_hash),
            top_visplane_used: false,
        }
    }

    // Add an existing visplane to the context and create a new one
    fn flush(&mut self, context: &mut Context) {
        if self.bottom_visplane_used {
            context.visplanes.push(self.bottom_visplane.clone());

            self.bottom_visplane = Visplane::new(self.floor_texture_hash);
            self.bottom_visplane_used = false;
        }

        if self.top_visplane_used {
            context.visplanes.push(self.top_visplane.clone());

            self.top_visplane = Visplane::new(self.ceiling_texture_hash);
            self.top_visplane_used = false;
        }
    }

    // Add a point to the bottom visplane
    fn add_bottom_point(&mut self, x: i16, top_y: i16, bottom_y: i16) {
        if !self.bottom_visplane_used {
            self.bottom_visplane.left = x;
        }

        self.bottom_visplane.right = x;

        self.bottom_visplane_used = true;
        self.bottom_visplane.top[x as usize] = top_y;
        self.bottom_visplane.bottom[x as usize] = bottom_y;
    }

    // Add a point to the top visplane
    fn add_top_point(&mut self, x: i16, top_y: i16, bottom_y: i16) {
        if !self.top_visplane_used {
            self.top_visplane.left = x;
        }

        self.top_visplane.right = x;

        self.top_visplane_used = true;
        self.top_visplane.top[x as usize] = top_y;
        self.top_visplane.bottom[x as usize] = bottom_y;
    }
}

fn check_sidedef_non_vertical_line_bounds(line: &SdlLine) {
    if line.start.x < 0 || line.start.x >= SCREEN_WIDTH as i32 {
        panic!("Invalid line start x: {}", line.start.x);
    }

    if line.end.x < 0 || line.end.x >= SCREEN_WIDTH as i32 {
        panic!("Invalid line end x: {}", line.end.x);
    }
}

// Process a part of a sidedef.
// This may involve drawing it, but might also involte processing occlusions and visplanes.
fn process_sidedef(
    game: &mut Game,
    context: &mut Context,
    bottom: &SdlLine,
    top: &SdlLine,
    floor_texture_hash: i16,
    ceiling_texture_hash: i16,
    is_whole_sidedef: bool, // For occlusion & visplane processing
    is_lower_wall: bool,    // For portals: the rendered piece of wall
    is_upper_wall: bool,    // For portals: the rendered piece of wall
) {
    // Do some sanity checks
    if bottom.start.x != top.start.x || bottom.end.x != top.end.x {
        panic!(
            "Wall start not vertical: {} vs {} or {} vs {}",
            &bottom.start.x, &top.start.x, &bottom.end.x, &top.end.x,
        );
    }

    check_sidedef_non_vertical_line_bounds(&bottom);
    check_sidedef_non_vertical_line_bounds(&top);

    if bottom.start.x == bottom.end.x || top.start.x == top.end.x {
        // TODO: it looks like some sidedefs get clipped entirely to the
        // edges of the viewport. This should not be happening.
        return;
    }

    // Loop from the left x to the right x, calculating the y screen coordinates
    // for the bottom and top.
    let bottom_delta = (bottom.start.y as f32 - bottom.end.y as f32)
        / (bottom.start.x as f32 - bottom.end.x as f32);
    let top_delta =
        (top.start.y as f32 - top.end.y as f32) / (top.start.x as f32 - top.end.x as f32);

    let mut vis_planes = SidedefVisPlanes::new(floor_texture_hash, ceiling_texture_hash);

    // Does the wall from from floor to ceiling?
    let is_full_height_wall = !is_lower_wall && !is_upper_wall && !is_whole_sidedef;

    for x in bottom.start.x as i16..bottom.end.x as i16 + 1 {
        if !context.hor_ocl[x as usize] {
            // Calculate top and bottom of the line
            let bottom_y =
                (bottom.start.y as f32 + (x as f32 - bottom.start.x as f32) * bottom_delta) as i16;
            let top_y = (top.start.y as f32 + (x as f32 - top.start.x as f32) * top_delta) as i16;

            // Is the line occluded?
            let floor_ver_ocl = context.floor_ver_ocl[x as usize];
            let ceiling_ver_ocl = context.ceiling_ver_ocl[x as usize];

            // Clip to non-occluded region (if any)
            let mut clipped_bottom_y = min(floor_ver_ocl, bottom_y);
            let mut clipped_top_y = max(ceiling_ver_ocl, top_y);

            clipped_bottom_y = min(SCREEN_HEIGHT as i16 - 1, clipped_bottom_y);
            clipped_top_y = max(0, clipped_top_y);

            let in_ver_clipped_area = clipped_bottom_y > clipped_top_y;

            // The line isn't occluded. Draw it.

            // Draw the vertical line unless it's transparent
            // The middle wall isn't rendered, it's only used to create visplanes.
            if !is_whole_sidedef && in_ver_clipped_area {
                game.canvas
                    .draw_line(
                        Point::new(x as i32, clipped_bottom_y as i32),
                        Point::new(x as i32, clipped_top_y as i32),
                    )
                    .unwrap();
            }

            if in_ver_clipped_area && (is_full_height_wall || is_whole_sidedef) {
                let mut visplane_added = false;
                // Process bottom visplane
                if clipped_bottom_y < floor_ver_ocl {
                    if clipped_bottom_y != SCREEN_HEIGHT as i16 - 1 {
                        vis_planes.add_bottom_point(x, clipped_bottom_y, floor_ver_ocl);
                        visplane_added = true;
                    }
                }

                if clipped_top_y > ceiling_ver_ocl {
                    if clipped_top_y != -1 {
                        vis_planes.add_top_point(x, ceiling_ver_ocl, clipped_top_y);
                        visplane_added = true;
                    }
                }

                if !visplane_added {
                    // Line is occluded, flush visplanes
                    vis_planes.flush(context);
                }
            } else if !in_ver_clipped_area
                && (is_full_height_wall || is_whole_sidedef)
                && floor_ver_ocl > ceiling_ver_ocl
            {
                // The sidedef is occluded. However, there is still is a vertical
                // unoccluded gap. Fill it with the floor/ceiling texture belonging to
                // the sidedef. This is rare, but happens e.g. in doom1 e1m1 when in
                // the hidden ahrea going down the stairs to the outside area.

                if bottom_y <= ceiling_ver_ocl {
                    vis_planes.add_bottom_point(x, ceiling_ver_ocl, floor_ver_ocl);

                    // Occlude the entire vertical line
                    context.occlude_vertical_line(x);
                }

                if top_y >= floor_ver_ocl {
                    vis_planes.add_top_point(x, ceiling_ver_ocl, floor_ver_ocl);

                    // Occlude the entire vertical line
                    context.occlude_vertical_line(x);
                }
            }

            if in_ver_clipped_area && is_whole_sidedef {
                context.floor_ver_ocl[x as usize] = clipped_bottom_y;
                context.ceiling_ver_ocl[x as usize] = clipped_top_y;
            }

            // Update vertical occlusions
            if in_ver_clipped_area && is_lower_wall {
                context.floor_ver_ocl[x as usize] = clipped_top_y;
            }

            if in_ver_clipped_area && is_upper_wall {
                context.ceiling_ver_ocl[x as usize] = clipped_bottom_y;
            }
        } else {
            // Line is occluded, flush visplanes
            vis_planes.flush(context);
        }

        if is_full_height_wall {
            // A vertical line occludes everything behind it
            context.occlude_vertical_line(x);
        }
    }

    vis_planes.flush(context);
}

// Draw a seg
fn render_seg(game: &mut Game, context: &mut Context, seg: &Seg) {
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

    let is_two_sided = linedef.flags & Flags::TWOSIDED != 0;

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
        .set_draw_color(WALL_COLORS[seg.id as usize % WALL_COLORS.len()]);

    // Draw the non-vertial lines for all parts of the wall
    let player_height = &game.player_floor_height + PLAYER_HEIGHT;

    // Draw the floor & ceiling lines
    let floor = make_sidedef_non_vertical_line(&clipped_line, floor_height - player_height);

    // We are facing the non-rendered side of the segment.
    if floor.start.x > floor.end.x {
        return;
    }

    let ceiling = make_sidedef_non_vertical_line(&clipped_line, ceiling_height - player_height);

    // portal_bottom is the optional top of the lower texture
    let portal_bottom = if let Some(portal_bottom_height) = opt_portal_bottom_height {
        Some(make_sidedef_non_vertical_line(
            &clipped_line,
            portal_bottom_height - player_height,
        ))
    } else {
        None
    };

    // portal_top is the optional bottom of the upper texture
    let portal_top = if let Some(portal_top_height) = opt_portal_top_height {
        Some(make_sidedef_non_vertical_line(
            &clipped_line,
            portal_top_height - player_height,
        ))
    } else {
        None
    };

    // We now have all the non-vertical lines, draw the walls in between them.
    if !is_two_sided {
        // Draw a solid wall's middle texture, floor to ceiling

        process_sidedef(
            game,
            context,
            &floor,
            &ceiling,
            front_sector.floor_texture_hash,
            front_sector.ceiling_texture_hash,
            false,
            false,
            false,
        );
    } else {
        // Draw a portal's lower and upper textures (if present)

        // Process the portal's bounds without drawing it
        process_sidedef(
            game,
            context,
            &floor,
            &ceiling,
            front_sector.floor_texture_hash,
            front_sector.ceiling_texture_hash,
            true, // Only process occlusions and visplanes
            false,
            false,
        );

        // Process the lower wall
        if let Some(portal_bottom) = portal_bottom.clone() {
            process_sidedef(
                game,
                context,
                &floor,
                &portal_bottom,
                front_sector.floor_texture_hash,
                front_sector.ceiling_texture_hash,
                false,
                true,
                false,
            );
        }

        // Process the upper wall
        if let Some(portal_top) = portal_top.clone() {
            process_sidedef(
                game,
                context,
                &portal_top,
                &ceiling,
                front_sector.floor_texture_hash,
                front_sector.ceiling_texture_hash,
                false,
                false,
                true,
            );
        }
    }
}

// Render all segs in a subsector
fn render_subsector(game: &mut Game, context: &mut Context, subsector: &SubSector) {
    for seg in &subsector.segs {
        render_seg(game, context, &seg);
    }
}

// Recurse through the BSP tree, drawing the subsector leaves
// The BSP algorithm guarantees that the subsectors are visited front to back.
fn render_node(game: &mut Game, context: &mut Context, node: &Rc<Node>) {
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
            render_node(game, context, &node);
        }
        NodeChild::SubSector(subsector) => {
            render_subsector(game, context, &subsector);
        }
    }

    // TODO: Use the bounding box and only recurse into the back of the split
    // if the player view intersects with it.
    match back_child {
        NodeChild::Node(node) => {
            render_node(game, context, &node);
        }
        NodeChild::SubSector(subsector) => {
            render_subsector(game, context, &subsector);
        }
    }
}

pub fn render_map(game: &mut Game) {
    let mut context = Context {
        hor_ocl: [false; SCREEN_WIDTH as usize],
        floor_ver_ocl: [SCREEN_HEIGHT as i16; SCREEN_WIDTH as usize],
        ceiling_ver_ocl: [-1; SCREEN_WIDTH as usize],
        visplanes: Vec::new(),
    };

    let root_node = Rc::clone(&game.map.root_node);
    render_node(game, &mut context, &root_node);

    draw_visplanes(game, &context);
}
