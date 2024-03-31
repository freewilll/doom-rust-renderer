use sdl2::pixels::Color;

use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Pixels {
    pub pixels: Vec<u8>, // The width * height pixels int the frame
}

impl Pixels {
    pub fn new() -> Pixels {
        Pixels {
            pixels: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize],
        }
    }

    #[allow(dead_code)]
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
