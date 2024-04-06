use core::cmp::Ordering;
use sdl2::pixels::Color;
use std::rc::Rc;

use super::clipped_line::ClippedLine;
use super::pixels::Pixels;

use crate::graphics::{Bitmap, Palette};

#[derive(PartialEq)]
pub enum BitmapRenderState {
    SolidSeg,    // Already drawn solid wall, only used for clipping map objects.
    TwoSidedSeg, // A portal. Must be drawn behind may objects. Also used for clipping map objects.
    DrawnSeg,    // A two sided portal that's already drawn
    MapObject,   // Is a map object
}

pub struct BitmapColumn {
    pub x: i32,                // The x coordinate in screen coordinate
    pub clipped_top_y: i32,    // The y region to draw in screen coordinates
    pub clipped_bottom_y: i32, // The y region to draw in screen coordinates
    pub bottom_y: i32,         // Full vertical line in screen coordinates
    pub top_y: i32,            // Full vertical line in screen coordinates
}

// Insane amount of context that is needed to call render_vertical_bitmap_line
// and do map object clipping.
pub struct BitmapRender {
    pub state: BitmapRenderState,   // Usage and if it's already been drawn
    bitmap: Option<Rc<Bitmap>>, // The texture or picture's bitmap, None if this is a non-rendered portal
    light_level: i16,           // Sector light level
    pub clipped_line: ClippedLine, // The clipped line in viewport coordinates
    start_x: i32,               // The clipped line x start in screen coordinates
    end_x: i32,                 // The clipped line x end in screen coordinates
    bottom_height: f32,         // The (potentially not-drawn) bottom in viewport coordinates
    top_height: f32,            // The (potentially not-drawn) top in viewport coordinates
    offset_x: i16,              // Texture offset in viewport coordinates
    offset_y: i16,              // Texture offset in viewport coordinates
    pub extends_to_bottom: bool, // Used to clip map objects against solid walls
    pub extends_to_top: bool,   // Used to clip map objects against solid walls
    pub draw_ceiling: bool,     // Set to false in a special case for sky texture
    pub columns: Vec<BitmapColumn>, // The columns
}

impl BitmapRender {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state: BitmapRenderState,   // The state
        bitmap: Option<Rc<Bitmap>>, // The texture or picture's bitmap, None if this is a non-rendered portal
        light_level: i16,           // Sector light level
        clipped_line: ClippedLine,  // The clipped line in viewport coordinates
        start_x: i32,               // The clipped line x start in screen coordinates
        end_x: i32,                 // The clipped line x end in screen coordinates
        bottom_height: f32,         // The (potentially not-drawn) bottom in viewport coordinates
        top_height: f32,            // The (potentially not-drawn) top in viewport coordinates
        offset_x: i16,              // Texture offset in viewport coordinates
        offset_y: i16,              // Texture offset in viewport coordinates
        extends_to_bottom: bool,    // Used to clip things against solid walls
        extends_to_top: bool,       // Used to clip things against solid walls
        draw_ceiling: bool,         // Set to false in a special case for sky texture
    ) -> BitmapRender {
        BitmapRender {
            state,
            bitmap,
            light_level,
            clipped_line,
            start_x,
            end_x,
            bottom_height,
            top_height,
            offset_x,
            offset_y,
            extends_to_bottom,
            extends_to_top,
            draw_ceiling,
            columns: vec![],
        }
    }

    pub fn add_column(
        &mut self,
        x: i16,                // The x coordinate in screen coordinate
        clipped_top_y: i16,    // The y region to draw in screen coordinates
        clipped_bottom_y: i16, // The y region to draw in screen coordinates
        bottom_y: i16,         // Full vertical line in screen coordinates
        top_y: i16,            // Full vertical line in screen coordinates
    ) {
        self.columns.push(BitmapColumn {
            x: x.into(),
            clipped_top_y: clipped_top_y.into(),
            clipped_bottom_y: clipped_bottom_y.into(),
            bottom_y: bottom_y.into(),
            top_y: top_y.into(),
        });
    }

    pub fn render(&mut self, pixels: &mut Pixels, palette: &Palette) {
        // Bail if already rendered
        if self.state == BitmapRenderState::SolidSeg || self.state == BitmapRenderState::DrawnSeg {
            return;
        }

        if let Some(bitmap) = &self.bitmap {
            for column in &self.columns {
                render_vertical_bitmap_line(
                    pixels,
                    palette,
                    bitmap,
                    self.light_level,
                    &self.clipped_line,
                    self.start_x,
                    self.end_x,
                    self.bottom_height,
                    self.top_height,
                    self.offset_x,
                    self.offset_y,
                    column.x,
                    column.clipped_bottom_y,
                    column.clipped_top_y,
                    column.bottom_y,
                    column.top_y,
                );
            }
        }

        // Note: this differs a bit from Doom which keeps track of which columns
        // are drawn. Here, an entire seg is either drawn or not.
        self.state = BitmapRenderState::DrawnSeg;
    }
}

impl Ord for BitmapRender {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_i16 = self.clipped_line.line.start.x as i16;
        let other_i16 = other.clipped_line.line.start.x as i16;
        self_i16.cmp(&other_i16)
    }
}

impl PartialOrd for BitmapRender {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BitmapRender {
    fn eq(&self, other: &Self) -> bool {
        self.clipped_line.line.start.x == other.clipped_line.line.start.x
    }
}

impl Eq for BitmapRender {}

pub fn diminish_color(color: &Color, light_level: i16, distance: i16) -> Color {
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
        (color.r as f32 * factor) as u8,
        (color.g as f32 * factor) as u8,
        (color.b as f32 * factor) as u8,
    )
}

// Draw a vertical line of a texture
// See 5.12.5 Perspective-Correct Texture Mapping in the game engine black book
#[allow(clippy::too_many_arguments)]
pub fn render_vertical_bitmap_line(
    pixels: &mut Pixels,
    palette: &Palette,
    bitmap: &Bitmap,            // The texture or picture's bitmap
    light_level: i16,           // Sector light level
    clipped_line: &ClippedLine, // The clipped line in viewport coordinates
    start_x: i32,               // The clipped line x start in screen coordinates
    end_x: i32,                 // The clipped line x end in screen coordinates
    bottom_height: f32,         // The (potentially not-drawn) bottom in viewport coordinates
    top_height: f32,            // The (potentially not-drawn) top in viewport coordinates
    offset_x: i16,              // Texture offset in viewport coordinates
    offset_y: i16,              // Texture offset in viewport coordinates
    x: i32,                     // The x coordinate in screen coordinate
    clipped_bottom_y: i32,      // The y region to draw in screen coordinates
    clipped_top_y: i32,         // The y region to draw in screen coordinates
    bottom_y: i32,              // Full vertical line in screen coordinates
    top_y: i32,                 // Full vertical line in screen coordinates
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
    tx += clipped_line.start_offset as i16 + offset_x;
    if tx < 0 {
        tx += bitmap.width * (1 - tx / bitmap.width)
    }
    tx %= bitmap.width;

    // z coordinate of column in world coordinates
    let z = (((1.0 - ax) + ax) / ((1.0 - ax) * (1.0 / uz0) + ax * (1.0 / uz1))) as i16;

    for y in clipped_top_y..clipped_bottom_y + 1 {
        // Calculate texture y
        // A simple linear interpolation will do; the x distance is not a factor
        let ay = (y - top_y) as f32 / (bottom_y - top_y) as f32;
        let mut ty = (bitmap.height as f32 + (1.0 - ay) * uy0 + ay * uy1) as i16;

        ty += offset_y;
        if ty < 0 {
            ty += bitmap.height * (1 - ty / bitmap.height)
        }
        ty %= bitmap.height;

        if let Some(color_value) = bitmap.pixels[ty as usize][tx as usize] {
            let color = palette.colors[color_value as usize];
            let diminished_color = diminish_color(&color, light_level, z);

            pixels.set(x as usize, y as usize, &diminished_color);
        }
    }
}
