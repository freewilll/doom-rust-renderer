use sdl2::pixels::Color;
use std::f32::consts::PI;
use std::rc::Rc;

use super::bitmap_render::diminish_color;
use super::constants::{
    ASPECT_RATIO_CORRECTION, CAMERA_FOCUS_X, CAMERA_FOCUS_Y, GAME_CAMERA_FOCUS_X, PLAYER_EYE_HEIGHT,
};
use super::pixels::Pixels;
use crate::flats::{Flat, FLAT_SIZE};
use crate::game::{Player, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::palette::Palette;
use crate::textures::Texture;
use crate::vertexes::Vertex;

#[derive(Debug, Clone)]
pub struct Visplane {
    // Describes a floor or ceiling area bounded by vertical left and right lines.
    pub flat: Rc<Flat>,                       // The image
    pub height: i16,                          // Height of the floor/ceiling
    pub light_level: i16,                     // Light level
    pub left: i16,                            // Minimum x coordinate
    pub right: i16,                           // Maximum x coordinate
    pub top: [i16; SCREEN_WIDTH as usize],    // Top line
    pub bottom: [i16; SCREEN_WIDTH as usize], // Bottom line
}

impl Visplane {
    pub fn new(flat: &Rc<Flat>, height: i16, light_level: i16) -> Visplane {
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

            if let Some(color_value) = sky_texture.bitmap.pixels[ty as usize][tx as usize] {
                let color = palette.colors[color_value as usize];
                pixels.set(x as usize, y as usize, &color);
            }
        }
    }
}

pub fn draw_visplane(
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

        // Don 't draw one pixel visplanes; they look like ugly solid horizontal lines
        if bottom - top <= 1 {
            continue;
        }

        for y in top..bottom + 1 {
            // x and y are in screen coordinates. We need to go backwards all the way
            // to world coordinates.

            // Transform to viewport coordinates (v prefix) (the reverse of make_sidedef_non_vertical_line)
            let vx = (CAMERA_FOCUS_X - x as f32) / ASPECT_RATIO_CORRECTION;
            let vy = CAMERA_FOCUS_Y - y as f32;

            // Inverse perspective transform to world coordinates (w prefix)
            let wz = visplane.height as f32 - player.floor_height - PLAYER_EYE_HEIGHT;
            let wx = GAME_CAMERA_FOCUS_X * wz / vy as f32;
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
