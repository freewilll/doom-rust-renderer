use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::fmt;

use crate::palette::Palette;
use crate::vertexes::Vertex;

#[derive(Clone)]
pub struct Bitmap {
    pub width: i16,                   // Width
    pub height: i16,                  // Height
    pub pixels: Vec<Vec<Option<u8>>>, // Grid of colormap indexes or None if transparent
}

impl Bitmap {
    pub fn new(width: i16, height: i16, pixels: Vec<Vec<Option<u8>>>) -> Bitmap {
        Bitmap {
            width,
            height,
            pixels,
        }
    }

    // Draw the bitmap to the top-left corner
    #[allow(dead_code)]
    pub fn test_flat_draw(&self, canvas: &mut Canvas<Window>, palette: &Palette, offset: &Vertex) {
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        let rect = Rect::new(
            offset.x as i32,
            offset.y as i32,
            self.width as u32 * 4,
            self.height as u32 * 4,
        );
        canvas.fill_rect(rect).unwrap();

        for x in 0..self.width as usize {
            for y in 0..self.height as usize {
                if let Some(value) = self.pixels[y][x] {
                    let color = palette.colors[value as usize];
                    canvas.set_draw_color(color);
                    let rect = Rect::new(
                        offset.x as i32 + x as i32 * 4,
                        offset.y as i32 + y as i32 * 4,
                        4,
                        4,
                    );
                    canvas.fill_rect(rect).unwrap();
                }
            }
        }
    }
}

impl fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bitmap: dimensions: {} x {}", self.width, self.height,)
    }
}
