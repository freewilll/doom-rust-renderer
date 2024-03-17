use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::cmp::{max, min};
use std::f32::consts::PI;
use std::rc::Rc;

use crate::flats::{Flat, Flats, FLAT_SIZE};
use crate::game::{Player, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::geometry::Line;
use crate::linedefs::Flags;
use crate::map::Map;
use crate::nodes::{Node, NodeChild};
use crate::palette::Palette;
use crate::segs::Seg;
use crate::sidedefs::Sidedef;
use crate::subsectors::SubSector;
use crate::textures::{Texture, Textures};
use crate::vertexes::Vertex;

const PLAYER_HEIGHT: f32 = 41.0;

// The game ran on 320x200 but ended up on monitors with squarepixels and  320x240
// https://doomwiki.org/wiki/Aspect_ratio#:~:text=it%20was%20wide.-,Design%20of%20graphics,to%20this%20hardware%20video%20mode.
pub const ASPECT_RATIO_CORRECTION: f32 = 200.0 / 240.0;

// Do the perspetive transformation using a more broad screen then the
// actual screen. This is transformed back by the caller. The end result
// is everything being shown on the screen as it would have on the original
// VGA screens.
pub const GAME_SCREEN_WIDTH: f32 = SCREEN_WIDTH as f32 / ASPECT_RATIO_CORRECTION;
pub const GAME_CAMERA_FOCUS: f32 = GAME_SCREEN_WIDTH as f32 / 2.0 as f32;

pub const CAMERA_FOCUS: f32 = SCREEN_WIDTH as f32 / 2.0;

pub struct Renderer<'a> {
    pixels: &'a mut Pixels,
    map: &'a Map,
    textures: &'a mut Textures,
    sky_texture: Rc<Texture>,
    flats: &'a mut Flats,
    palette: &'a mut Palette,
    player: &'a Player,
    ticks: f32,
    hor_ocl: [bool; SCREEN_WIDTH as usize], // Horizontal occlusions
    floor_ver_ocl: [i16; SCREEN_WIDTH as usize], // Vertical occlusions for the floor
    ceiling_ver_ocl: [i16; SCREEN_WIDTH as usize], // Vertical occlusions for the ceiling
    vis_planes: Vec<Visplane>,
}

#[derive(Debug, Clone)]
struct Visplane {
    // Describes a floor or ceiling area bounded by vertical left and right lines.
    flat: Rc<Flat>,                       // The image
    height: i16,                          // Height of the floor/ceiling
    light_level: i16,                     // Light level
    left: i16,                            // Minimum x coordinate
    right: i16,                           // Maximum x coordinate
    top: [i16; SCREEN_WIDTH as usize],    // Top line
    bottom: [i16; SCREEN_WIDTH as usize], // Bottom line
}

impl Visplane {
    fn new(flat: &Rc<Flat>, height: i16, light_level: i16) -> Visplane {
        Visplane {
            flat: Rc::clone(&flat),
            height,
            light_level,
            left: -1,
            right: -1,
            top: [0; SCREEN_WIDTH as usize],
            bottom: [0; SCREEN_WIDTH as usize],
        }
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

struct ClippedLine {
    line: Line,
    start_offset: f32, // The amount the line was clipped by at the start/left end
}

pub struct Pixels {
    pub pixels: Vec<u8>, // The width * height pixels int the frame
}

impl Pixels {
    pub fn new() -> Pixels {
        Pixels {
            pixels: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
        }
    }

    pub fn clear(&mut self) {
        self.pixels.iter_mut().for_each(|x| *x = 0);
    }

    // Set a single pixel
    pub fn set(&mut self, x: usize, y: usize, color: &Color) {
        if x >= SCREEN_WIDTH as usize || y > SCREEN_HEIGHT as usize {
            return;
        }

        self.pixels[3 * (y as usize * SCREEN_WIDTH as usize + x as usize) + 0] = color.r;
        self.pixels[3 * (y as usize * SCREEN_WIDTH as usize + x as usize) + 1] = color.g;
        self.pixels[3 * (y as usize * SCREEN_WIDTH as usize + x as usize) + 2] = color.b;
    }

    // Draw a vertical line
    pub fn draw_vertical_line(&mut self, x: i32, top: i32, bottom: i32, color: &Color) {
        if x <= 0 || x >= SCREEN_WIDTH as i32 {
            return;
        }

        for y in top..bottom + 1 {
            if y < 0 || y >= SCREEN_HEIGHT as i32 {
                continue;
            }

            self.pixels[3 * (y as usize * SCREEN_WIDTH as usize + x as usize) + 0] = color.r;
            self.pixels[3 * (y as usize * SCREEN_WIDTH as usize + x as usize) + 1] = color.g;
            self.pixels[3 * (y as usize * SCREEN_WIDTH as usize + x as usize) + 2] = color.b;
        }
    }
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

    Vertex::new(GAME_CAMERA_FOCUS * x / z, GAME_CAMERA_FOCUS * y / z)
}

fn clip_to_viewport(line: &Line) -> Option<ClippedLine> {
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
        start_offset: start_offset,
    };

    Some(clipped_line)
}

// Make the slanted non-vertical line for a sidedef.
fn make_sidedef_non_vertical_line(line: &Line, height: f32) -> SdlLine {
    let mut transformed_start = perspective_transform(&line.start, height);
    let mut transformed_end = perspective_transform(&line.end, height);

    // Convert the in-game coordinates that are broad into the more narrow
    // screen x coordinates
    transformed_start.x *= ASPECT_RATIO_CORRECTION;
    transformed_end.x *= ASPECT_RATIO_CORRECTION;

    let mut screen_start = Point::new(
        ((CAMERA_FOCUS - &transformed_start.x) as i32).into(),
        ((CAMERA_FOCUS - &transformed_start.y) as i32).into(),
    );

    let mut screen_end = Point::new(
        ((CAMERA_FOCUS - &transformed_end.x) as i32).into(),
        ((CAMERA_FOCUS - &transformed_end.y) as i32).into(),
    );

    screen_start.x = screen_start.x.min(SCREEN_WIDTH as i32 - 1);
    screen_end.x = screen_end.x.min(SCREEN_WIDTH as i32 - 1);

    SdlLine::new(&screen_start, &screen_end)
}

// Keep track of the visplane state while processing a sidedef
struct SidedefVisPlanes {
    light_level: i16,
    floor_flat: Rc<Flat>,
    ceiling_flat: Rc<Flat>,
    floor_height: i16,
    ceiling_height: i16,
    bottom_visplane: Visplane,
    top_visplane: Visplane,
    bottom_visplane_used: bool,
    top_visplane_used: bool,
}

impl SidedefVisPlanes {
    fn new(
        light_level: i16,
        floor_flat: &Rc<Flat>,
        ceiling_flat: &Rc<Flat>,
        floor_height: i16,
        ceiling_height: i16,
    ) -> SidedefVisPlanes {
        SidedefVisPlanes {
            light_level,
            floor_flat: Rc::clone(floor_flat),
            ceiling_flat: Rc::clone(ceiling_flat),
            floor_height: floor_height,
            ceiling_height: ceiling_height,
            bottom_visplane: Visplane::new(floor_flat, floor_height, light_level),
            bottom_visplane_used: false,
            top_visplane: Visplane::new(ceiling_flat, ceiling_height, light_level),
            top_visplane_used: false,
        }
    }

    // Add an existing visplane and create a new one
    fn flush(&mut self, renderer: &mut Renderer) {
        if self.bottom_visplane_used {
            renderer.vis_planes.push(self.bottom_visplane.clone());

            self.bottom_visplane =
                Visplane::new(&self.floor_flat, self.floor_height, self.light_level);
            self.bottom_visplane_used = false;
        }

        if self.top_visplane_used {
            renderer.vis_planes.push(self.top_visplane.clone());

            self.top_visplane =
                Visplane::new(&self.ceiling_flat, self.ceiling_height, self.light_level);
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

fn draw_sky(
    pixels: &mut Pixels,
    palette: &Palette,
    player: &Player,
    sky_texture: Rc<Texture>,
    visplane: &Visplane,
) {
    const SKY_TEXTURE_WIDTH: i16 = 256; // Corresponds with the 90-degree player view
    const SKY_TEXTURE_HEIGHT: i16 = 128;

    // Based on the player angle, calculate the x-offset into the sky texture
    // 90 degrees of player angle is one SKY_TEXTURE_WIDTH
    let mut tx_offset =
        (-SKY_TEXTURE_WIDTH as f32 * player.angle / (PI / 2.0)) as i16 + SKY_TEXTURE_WIDTH;
    if tx_offset < 0 {
        tx_offset += SKY_TEXTURE_WIDTH * (1 - tx_offset / SKY_TEXTURE_WIDTH);
    }

    for x in visplane.left..visplane.right + 1 {
        let top = visplane.top[x as usize].max(0);
        let bottom = visplane.bottom[x as usize].min(SCREEN_HEIGHT as i16 - 1);

        for y in top..bottom + 1 {
            let mut tx = (x as f32 * SKY_TEXTURE_WIDTH as f32 / SCREEN_WIDTH as f32) as i16;
            tx = (tx + tx_offset) % SKY_TEXTURE_WIDTH;

            let ty = (y as f32 * SKY_TEXTURE_HEIGHT as f32 / SCREEN_HEIGHT as f32) as i16;

            let color = palette.colors[sky_texture.pixels[ty as usize][tx as usize] as usize];

            pixels.set(x as usize, y as usize, &color);
        }
    }
}

fn diminish_color(color: &Color, light_level: i16, distance: i16) -> Color {
    let mut factor = light_level as f32 / 255.0; // Start with the sector light level

    // Reduce the light based on the distance
    // See r_plane.c
    // The factor below is based on a visual feel of how things look rather
    // then a calculation of what the actual doom code does.
    let dimishing_factor: f32 = 1.0 / (16.0 * 256.0);
    factor -= distance as f32 * dimishing_factor;
    if factor < 0.0 {
        factor = 0.0
    };

    Color::RGB(
        (color.r as f32 * factor as f32) as u8,
        (color.g as f32 * factor as f32) as u8,
        (color.b as f32 * factor as f32) as u8,
    )
}

fn draw_visplane(
    pixels: &mut Pixels,
    palette: &Palette,
    player: &Player,
    sky_texture: Rc<Texture>,
    visplane: &Visplane,
) {
    const DEBUG_DRAW_OUTLINE: bool = false;

    if visplane.flat.name.contains("SKY") {
        draw_sky(pixels, palette, player, Rc::clone(&sky_texture), visplane);
        return;
    }

    for x in visplane.left..visplane.right + 1 {
        let top = visplane.top[x as usize].max(0);
        let bottom = visplane.bottom[x as usize].min(SCREEN_HEIGHT as i16 - 1);

        for y in top..bottom + 1 {
            // x and y are in screen coordinates. We need to go backwards all the way
            // to world coordinates.

            // Transform to viewport coordinates (v prefix) (the reverse of make_sidedef_non_vertical_line)
            let vx = (CAMERA_FOCUS - x as f32) / ASPECT_RATIO_CORRECTION;
            let vy = CAMERA_FOCUS - y as f32;

            // Inverse perspective transform to world coordinates (w prefix)
            let wz = visplane.height as f32 - player.floor_height - PLAYER_HEIGHT;
            let wx = GAME_CAMERA_FOCUS * wz / vy as f32;
            let wy = wz * vx as f32 / vy as f32;

            // Translate and rotate to player view
            let rotated = Vertex::new(wx, wy).rotate(player.angle);

            let mut tx: i16 = rotated.x as i16 + player.position.x as i16;
            let mut ty: i16 = rotated.y as i16 + player.position.y as i16;

            tx = tx & (FLAT_SIZE - 1);
            ty = ty & (FLAT_SIZE - 1);

            let color = palette.colors[visplane.flat.pixels[ty as usize][tx as usize] as usize];
            let diminished_color = diminish_color(&color, visplane.light_level, wx as i16);

            pixels.set(x as usize, y as usize, &diminished_color);
        }
    }

    if DEBUG_DRAW_OUTLINE {
        let outline_color = Color::RGB(255, 255, 255);
        for x in visplane.left..visplane.right + 1 {
            let top = visplane.top[x as usize].max(0);
            let bottom = visplane.bottom[x as usize].min(SCREEN_HEIGHT as i16 - 1);

            pixels.set(x as usize, top as usize, &outline_color);
            pixels.set(x as usize, bottom as usize, &outline_color);
        }

        let left = visplane.left as i32;
        let top = visplane.top[left as usize].max(0) as i32;
        let bottom = visplane.bottom[left as usize].min(SCREEN_HEIGHT as i16 - 1) as i32;
        pixels.draw_vertical_line(left, top, bottom, &outline_color);

        let right = visplane.right as i32;
        let top = visplane.top[right as usize].max(0) as i32;
        let bottom = visplane.bottom[right as usize].min(SCREEN_HEIGHT as i16 - 1) as i32;
        pixels.draw_vertical_line(right, top, bottom, &outline_color);
    }
}

impl Renderer<'_> {
    pub fn new<'a>(
        pixels: &'a mut Pixels,
        map: &'a Map,
        textures: &'a mut Textures,
        sky_texture: Rc<Texture>,
        flats: &'a mut Flats,
        palette: &'a mut Palette,
        player: &'a Player,
        ticks: f32,
    ) -> Renderer<'a> {
        Renderer {
            pixels,
            map,
            textures,
            sky_texture,
            flats,
            palette,
            player,
            ticks,
            hor_ocl: [false; SCREEN_WIDTH as usize],
            floor_ver_ocl: [SCREEN_HEIGHT as i16; SCREEN_WIDTH as usize],
            ceiling_ver_ocl: [-1; SCREEN_WIDTH as usize],
            vis_planes: Vec::new(),
        }
    }

    fn draw_visplanes(&mut self) {
        for visplane in &self.vis_planes {
            draw_visplane(
                &mut self.pixels,
                &self.palette,
                &self.player,
                Rc::clone(&self.sky_texture),
                &visplane,
            );
        }
    }

    fn check_sidedef_non_vertical_line_bounds(&self, line: &SdlLine) {
        if line.start.x < 0 || line.start.x >= SCREEN_WIDTH as i32 {
            panic!("Invalid line start x: {}", line.start.x);
        }

        if line.end.x < 0 || line.end.x >= SCREEN_WIDTH as i32 {
            panic!("Invalid line end x: {}", line.end.x);
        }
    }

    fn occlude_vertical_line(&mut self, x: i16) {
        self.hor_ocl[x as usize] = true;
        self.floor_ver_ocl[x as usize] = SCREEN_HEIGHT as i16 / 2;
        self.ceiling_ver_ocl[x as usize] = SCREEN_HEIGHT as i16 / 2;
    }

    // Draw a vertical line of a texture
    // See 5.12.5 Perspective-Correct Texture Mapping in the game engine black book
    fn render_vertical_texture_line(
        &mut self,
        sidedef: &Sidedef,          // The sidedef
        texture: &Texture,          // The texture
        light_level: i16,           // Sector light level
        clipped_line: &ClippedLine, // The clipped line in viewport coordinates
        start_x: i32,               // The clipped line x start in screen coordinates
        end_x: i32,                 // The clipped line x end in screen coordinates
        x: i32,                     // The x coordinate in screen coordinate
        clipped_top_y: i32,         // The y region to draw in screen coordinates
        clipped_bottom_y: i32,      // The y region to draw in screen coordinates
        bottom_height: f32,         // The (potentially not-drawn) bottom in viewport coordinates
        top_height: f32,            // The (potentially not-drawn) top in viewport coordinates
        bottom_y: i32,              // Full vertical line in screen coordinates
        top_y: i32,                 // Full vertical line in screen coordinates
        offset_x: i16,              // Texture offset in viewport coordinates
        offset_y: i16,              // Texture offset in viewport coordinates
    ) {
        let len = clipped_line.line.length();

        let (ux0, ux1) = (0.0, len);
        let (uy0, uy1) = (0.0, top_height - bottom_height);
        let (uz0, uz1) = (clipped_line.line.start.x, clipped_line.line.end.x);

        // Determine texture x tx. This only needs doing once outside
        // of the y-loop.
        let ax = (x - start_x) as f32 / (end_x - start_x) as f32;
        let mut tx = (((1.0 - ax) * (ux0 / uz0) + ax * (ux1 / uz1))
            / ((1.0 - ax) * (1.0 / uz0) + ax * (1.0 / uz1))) as i16;
        tx = clipped_line.start_offset as i16 + offset_x + sidedef.x_offset as i16 + tx;
        if tx < 0 {
            tx += texture.width * (1 - tx / texture.width)
        }
        tx = tx % texture.width;

        // z coordinate of column in world coordinates
        let z = (((1.0 - ax) * (uz0 / uz0) + ax * (uz1 / uz1))
            / ((1.0 - ax) * (1.0 / uz0) + ax * (1.0 / uz1))) as i16;

        for y in clipped_top_y..clipped_bottom_y + 1 {
            // Calculate texture y
            // A simple linear interpolation will do; the x distance is not a factor
            let ay = (y - top_y) as f32 / (bottom_y - top_y) as f32;
            let mut ty = (texture.height as f32 + (1.0 - ay) * uy0 + ay * uy1) as i16;

            ty = offset_y + sidedef.y_offset as i16 + ty;
            if ty < 0 {
                ty += texture.height * (1 - ty / texture.height)
            }
            ty = ty % texture.height;

            let color = self.palette.colors[texture.pixels[ty as usize][tx as usize] as usize];
            let diminished_color = diminish_color(&color, light_level, z);

            self.pixels.set(x as usize, y as usize, &diminished_color);
        }
    }

    // Process a part of a sidedef.
    // This may involve drawing it, but might also involve processing occlusions and visplanes.
    fn process_sidedef(
        &mut self,
        clipped_line: &ClippedLine, // The clipped line in viewport coords
        sidedef: &Sidedef,          // The sidedef
        bottom_height: f32,         // Height of the bottom of the clipped line in viewport coords
        top_height: f32,            // Height of the top of the clipped line in viewport coords
        seg_offset: i16,            // Distance along linedef to start of seg
        offset_y: i32,              // Texture offset in viewport coords
        texture_name: &str,         // Optional texture
        light_level: i16,           // Sector light level
        floor_flat: &Rc<Flat>,      // Floor texture
        ceiling_flat: &Rc<Flat>,    // Ceiling texture
        floor_height: i16,          // Height of the floor
        ceiling_height: i16,        // Height of the ceiling
        is_whole_sidedef: bool,     // For occlusion & visplane processing
        is_lower_wall: bool,        // For portals: the rendered piece of wall
        is_upper_wall: bool,        // For portals: the rendered piece of wall
        draw_ceiling: bool,         // Set to false in a special case for sky texture
    ) {
        let bottom = make_sidedef_non_vertical_line(&clipped_line.line, bottom_height);
        let top = make_sidedef_non_vertical_line(&clipped_line.line, top_height);

        let texture = if !is_whole_sidedef && texture_name != "-" {
            Some(self.textures.get(texture_name))
        } else {
            None
        };

        // Do some sanity checks
        if bottom.start.x != top.start.x || bottom.end.x != top.end.x {
            panic!(
                "Wall start not vertical: {} vs {} or {} vs {}",
                &bottom.start.x, &top.start.x, &bottom.end.x, &top.end.x,
            );
        }

        // Catch division by zero, which happens if we're looking at the wall from
        // the side, dead on. In this case, there's nothing to see.
        if bottom.start.x as i16 == bottom.end.x as i16 || top.start.x as i16 == top.end.x as i16 {
            return;
        }

        self.check_sidedef_non_vertical_line_bounds(&bottom);
        self.check_sidedef_non_vertical_line_bounds(&top);

        // Loop from the left x to the right x, calculating the y screen coordinates
        // for the bottom and top.
        let bottom_delta = (bottom.start.y as f32 - bottom.end.y as f32)
            / (bottom.start.x as f32 - bottom.end.x as f32);
        let top_delta =
            (top.start.y as f32 - top.end.y as f32) / (top.start.x as f32 - top.end.x as f32);

        let mut sidedef_vis_planes = SidedefVisPlanes::new(
            light_level,
            floor_flat,
            ceiling_flat,
            floor_height,
            ceiling_height,
        );

        // Does the wall from from floor to ceiling?
        let is_full_height_wall = !is_lower_wall && !is_upper_wall && !is_whole_sidedef;

        for x in bottom.start.x as i16..bottom.end.x as i16 + 1 {
            if !self.hor_ocl[x as usize] {
                // Calculate top and bottom of the line
                let bottom_y = (bottom.start.y as f32
                    + (x as f32 - bottom.start.x as f32) * bottom_delta)
                    as i16;
                let top_y =
                    (top.start.y as f32 + (x as f32 - top.start.x as f32) * top_delta) as i16;

                // Is the line occluded?
                let floor_ver_ocl = self.floor_ver_ocl[x as usize];
                let ceiling_ver_ocl = self.ceiling_ver_ocl[x as usize];

                // Clip to non-occluded region (if any)
                let mut clipped_bottom_y = min(floor_ver_ocl, bottom_y);
                let mut clipped_top_y = max(ceiling_ver_ocl, top_y);

                clipped_bottom_y = min(SCREEN_HEIGHT as i16 - 1, clipped_bottom_y);
                clipped_top_y = max(0, clipped_top_y);

                // Include special case of clipped_bottom_y == clipped_top_y, which
                // takes care of zero-height sectors, e.g. sector 16 on the ourside
                // of the outside area in e1m1
                let in_ver_clipped_area = clipped_bottom_y >= clipped_top_y;

                // The line isn't occluded. Draw it.

                // Draw the vertical line unless it's transparent
                // The middle wall isn't rendered, it's only used to create visplanes.
                if !is_whole_sidedef && in_ver_clipped_area {
                    match texture {
                        None => {
                            // Draw a missing texture as white
                            self.pixels.draw_vertical_line(
                                x.into(),
                                clipped_top_y.into(),
                                clipped_bottom_y.into(),
                                &Color::RGB(255, 255, 255),
                            );
                        }
                        Some(ref texture) => {
                            self.render_vertical_texture_line(
                                &sidedef,
                                &texture,
                                light_level,
                                &clipped_line,
                                bottom.start.x,
                                bottom.end.x,
                                x.into(),
                                clipped_top_y.into(),
                                clipped_bottom_y.into(),
                                bottom_height,
                                top_height,
                                bottom_y.into(),
                                top_y.into(),
                                seg_offset,
                                offset_y as i16,
                            );
                        }
                    };
                }

                if in_ver_clipped_area && (is_full_height_wall || is_whole_sidedef) {
                    let mut visplane_added = false;
                    // Process bottom visplane
                    if clipped_bottom_y < floor_ver_ocl {
                        if clipped_bottom_y != SCREEN_HEIGHT as i16 - 1 {
                            sidedef_vis_planes.add_bottom_point(x, clipped_bottom_y, floor_ver_ocl);
                            visplane_added = true;
                        }
                    }

                    // Process top visplane
                    if draw_ceiling && clipped_top_y > ceiling_ver_ocl {
                        if clipped_top_y != -1 {
                            if draw_ceiling {
                                sidedef_vis_planes.add_top_point(x, ceiling_ver_ocl, clipped_top_y);
                            }
                            visplane_added = true;
                        }
                    }

                    if !visplane_added {
                        // Line is occluded, flush visplanes
                        sidedef_vis_planes.flush(self);
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
                        sidedef_vis_planes.add_bottom_point(x, ceiling_ver_ocl, floor_ver_ocl);

                        // Occlude the entire vertical line
                        self.occlude_vertical_line(x);
                    }

                    if draw_ceiling && top_y >= floor_ver_ocl {
                        if draw_ceiling {
                            sidedef_vis_planes.add_top_point(x, ceiling_ver_ocl, floor_ver_ocl);
                        }

                        // Occlude the entire vertical line
                        self.occlude_vertical_line(x);
                    }
                }

                if in_ver_clipped_area && is_whole_sidedef {
                    self.floor_ver_ocl[x as usize] = clipped_bottom_y;

                    if draw_ceiling {
                        self.ceiling_ver_ocl[x as usize] = clipped_top_y;
                    }
                }

                // Update vertical occlusions
                if in_ver_clipped_area && is_lower_wall {
                    self.floor_ver_ocl[x as usize] = clipped_top_y;
                }

                if in_ver_clipped_area && is_upper_wall {
                    self.ceiling_ver_ocl[x as usize] = clipped_bottom_y;
                }
            } else {
                // Line is occluded, flush visplanes
                sidedef_vis_planes.flush(self);
            }

            if is_full_height_wall {
                // A vertical line occludes everything behind it
                self.occlude_vertical_line(x);
            }
        }

        sidedef_vis_planes.flush(self);
    }

    // Draw a seg
    fn render_seg(&mut self, seg: &Seg) {
        // Get the linedef
        let linedef = &seg.linedef;

        // Get the sidedef(s)
        let (opt_front_sidedef, opt_back_sidedef) = if seg.direction {
            (&linedef.back_sidedef, &linedef.front_sidedef)
        } else {
            (&linedef.front_sidedef, &linedef.back_sidedef)
        };

        // Get the front sector (the one we're facing)
        let front_sidedef = match opt_front_sidedef {
            Some(s) => s,
            None => {
                // If there is no sidedef, then there is no wall
                return;
            }
        };

        let front_sector = &front_sidedef.sector;

        // Get the floor and ceiling height from the front sector
        let floor_height = front_sector.floor_height as f32;
        let mut ceiling_height = front_sector.ceiling_height as f32;

        // For portals, get the bottom and top heights by looking at the back
        // sector.
        let (opt_portal_bottom_height, mut opt_portal_top_height) = match opt_back_sidedef {
            Some(back_sidedef) => {
                let back_sector = &back_sidedef.sector;

                let opt_portal_bottom_height =
                    if back_sector.floor_height > front_sector.floor_height {
                        Some(back_sector.floor_height as f32)
                    } else {
                        None
                    };

                let opt_portal_top_height =
                    if back_sector.ceiling_height < front_sector.ceiling_height {
                        Some(back_sector.ceiling_height as f32)
                    } else {
                        None
                    };

                (opt_portal_bottom_height, opt_portal_top_height)
            }
            None => (None, None),
        };

        let is_two_sided = linedef.flags & Flags::TWOSIDED != 0;
        let top_is_unpegged = linedef.flags & Flags::DONTPEGTOP != 0;
        let bottom_is_unpegged = linedef.flags & Flags::DONTPEGBOTTOM != 0;

        // Transform the seg so that the player position and angle is transformed
        // away.

        let moved_start = &*seg.start_vertex - &self.player.position;
        let moved_end = &*seg.end_vertex - &self.player.position;

        let start = moved_start.rotate(-self.player.angle);
        let end = moved_end.rotate(-self.player.angle);

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

        if clipped_line.line.start.x < -0.01 {
            panic!(
                "Clipped line x < -0.01: {:?} player: {:?}",
                &clipped_line.line.start.x, &self.player.position
            );
        }

        // Draw the non-vertial lines for all parts of the wall
        let player_height = &self.player.floor_height + PLAYER_HEIGHT;

        // Check one line to ensure we're not facing the back of it
        let floor =
            make_sidedef_non_vertical_line(&clipped_line.line, floor_height - player_height);

        // We are facing the non-rendered side of the segment.
        if floor.start.x > floor.end.x {
            return;
        }

        let floor_flat = self
            .flats
            .get_animated(front_sector.floor_texture.as_str(), self.ticks);
        let ceiling_flat = self
            .flats
            .get_animated(front_sector.ceiling_texture.as_str(), self.ticks);

        let mut draw_ceiling = true;

        // If both the front and back sector are sky, then don't draw the top linedef
        // and don't draw the sky.
        // https://doomwiki.org/wiki/Sky_hack
        // This follows the gory details in r_segs.c
        if let Some(back_sidedef) = opt_back_sidedef {
            if front_sidedef.sector.ceiling_texture.contains("SKY")
                && back_sidedef.sector.ceiling_texture.contains("SKY")
            {
                opt_portal_top_height = None;
                ceiling_height = back_sidedef.sector.ceiling_height as f32;
                draw_ceiling = false;
            }
        }

        // All the transformations are done and the wall/portal is facing us.
        // Call the sidedef processor with the three parts of the wall/portal.
        // https://doomwiki.org/wiki/Texture_alignment
        if !is_two_sided {
            // Draw a solid wall's middle texture, floor to ceiling

            let offset_y = if bottom_is_unpegged {
                // Setting bottom_is_unpegged makes the texture located at the floor
                (floor_height - ceiling_height) as i32
            } else {
                // Default to the texture being locatd at the top
                0
            };

            // Draw the solid wall texture
            self.process_sidedef(
                &clipped_line,
                &front_sidedef,
                floor_height - player_height,
                ceiling_height - player_height,
                seg.offset,
                offset_y,
                &front_sidedef.middle_texture,
                front_sector.light_level,
                &floor_flat,
                &ceiling_flat,
                front_sector.floor_height,
                front_sector.ceiling_height,
                false,
                false,
                false,
                draw_ceiling,
            );
        } else {
            // Draw a portal's lower and upper textures (if present)

            // Process the portal's bounds without drawing it
            self.process_sidedef(
                &clipped_line,
                &front_sidedef,
                floor_height - player_height,
                ceiling_height - player_height,
                seg.offset,
                0,
                &front_sidedef.middle_texture,
                front_sector.light_level,
                &floor_flat,
                &ceiling_flat,
                front_sector.floor_height,
                front_sector.ceiling_height,
                true, // Only process occlusions and visplanes
                false,
                false,
                draw_ceiling,
            );

            // Process the lower texture
            if let Some(portal_bottom_height) = opt_portal_bottom_height {
                let offset_y = if bottom_is_unpegged {
                    // The lower texture starts at the highest floor
                    (ceiling_height - portal_bottom_height) as i32
                } else {
                    // The lower texture starts as if it started at the highest ceiling
                    0
                };

                self.process_sidedef(
                    &clipped_line,
                    &front_sidedef,
                    floor_height - player_height,
                    portal_bottom_height - player_height,
                    seg.offset,
                    offset_y,
                    &front_sidedef.lower_texture,
                    front_sector.light_level,
                    &floor_flat,
                    &ceiling_flat,
                    front_sector.floor_height,
                    front_sector.ceiling_height,
                    false,
                    true,
                    false,
                    draw_ceiling,
                );
            }

            // Process the upper texture
            if let Some(portal_top_height) = opt_portal_top_height {
                let offset_y = if top_is_unpegged {
                    // The upper texture starts at the ceiling
                    0
                } else {
                    // The upper texture starts at the lower ceiling
                    (portal_top_height - ceiling_height) as i32
                };

                self.process_sidedef(
                    &clipped_line,
                    &front_sidedef,
                    portal_top_height - player_height,
                    ceiling_height - player_height,
                    seg.offset,
                    offset_y,
                    &front_sidedef.upper_texture,
                    front_sector.light_level,
                    &floor_flat,
                    &ceiling_flat,
                    front_sector.floor_height,
                    front_sector.ceiling_height,
                    false,
                    false,
                    true,
                    draw_ceiling,
                );
            }
        }
    }

    // Render all segs in a subsector
    fn render_subsector(&mut self, subsector: &SubSector) {
        for seg in &subsector.segs {
            self.render_seg(&seg);
        }
    }

    // Recurse through the BSP tree, drawing the subsector leaves
    // The BSP algorithm guarantees that the subsectors are visited front to back.
    fn render_node(&mut self, node: &Rc<Node>) {
        let v1 = Vertex::new(node.x, node.y);
        let v2 = &v1 + &Vertex::new(node.dx, node.dy);

        let is_left = self.player.position.is_left_of_line(&Line::new(&v1, &v2));

        let (front_child, back_child) = if is_left {
            (&node.left_child, &node.right_child)
        } else {
            (&node.right_child, &node.left_child)
        };

        match front_child {
            NodeChild::Node(node) => {
                self.render_node(&node);
            }
            NodeChild::SubSector(subsector) => {
                self.render_subsector(&subsector);
            }
        }

        // TODO: Use the bounding box and only recurse into the back of the split
        // if the player view intersects with it.
        match back_child {
            NodeChild::Node(node) => {
                self.render_node(&node);
            }
            NodeChild::SubSector(subsector) => {
                self.render_subsector(&subsector);
            }
        }
    }

    pub fn render(&mut self) {
        let root_node = Rc::clone(&self.map.root_node);
        self.render_node(&root_node);

        self.draw_visplanes();
    }
}
