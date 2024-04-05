use std::cmp::{max, min};
use std::f32::consts::PI;
use std::rc::Rc;

use super::bitmap_render::{BitmapRender, BitmapRenderState};
use super::bsp::get_sector_from_vertex;
use super::constants::PLAYER_EYE_HEIGHT;
use super::misc::{clip_to_viewport, make_sidedef_non_vertical_line};
use super::pixels::Pixels;
use crate::game::{Player, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::geometry::Line;
use crate::info::StateId;
use crate::map::Map;
use crate::map_objects::MapObjects;
use crate::palette::Palette;
use crate::sprites::Sprites;
use crate::vertexes::Vertex;

// Draw map objects (aka things)
pub fn draw_map_objects(
    segs: &mut Vec<BitmapRender>,
    pixels: &mut Pixels,
    map_objects: &MapObjects,
    player: &Player,
    sprites: &Sprites,
    map: &Map,
    palette: &Palette,
) {
    // Loop over all map objects, prepare the bitmaps, transform and do
    // clipping.
    let mut map_object_bitmap_renders: Vec<BitmapRender> = Vec::new();

    for map_object in map_objects.objects.iter() {
        let map_object = map_object.borrow();

        if map_object.state.id == StateId::S_NULL {
            continue;
        }

        let sprite = &map_object.state.sprite;

        // Determine the rotation the player is facing the map object with. Rotations
        // are zero-indexed, so it looks like this:
        //        2
        //      3 | 1
        //       \|/
        //     4--*----> 0   Thing is facing this direction
        //       /|\
        //      5 | 7
        //        6

        // Some modulo & rounding acrobatics follow. Look away. this is ugly.
        // Find relative angle
        let mut angle = player.angle - map_object.angle - PI;

        // Add 22.5 degrees so that angles are rounded down to the nearest 45 degree angle
        angle += PI / 16.0;

        // Convert angle to range 0 to 2*pi
        angle %= 2.0 * PI;
        if angle < 0.0 {
            angle += 2.0 * PI;
        }
        angle %= 2.0 * PI;

        let rotation = (angle * 8.0 / (2.0 * PI)) as u8;

        let frame = map_object.state.frame;
        let picture = sprites.get_picture(sprite, frame, rotation);

        // Transform so that the player position and angle is transformed
        // away.
        let moved = &map_object.position - &player.position;
        let view_port_vertex = moved.rotate(-player.angle);

        let width = picture.bitmap.width;

        // The picture is always centered
        let start = &view_port_vertex - &Vertex::new(0.0, -width as f32 / 2.0_f32);
        let end = &view_port_vertex - &Vertex::new(0.0, width as f32 / 2.0_f32);

        let line = Line::new(&start, &end);

        let clipped_line = match clip_to_viewport(&line) {
            Some(clipped_line) => clipped_line,
            None => {
                continue;
            }
        };

        if clipped_line.line.start.x < -0.01 {
            panic!(
                "Clipped line x < -0.01: {:?} player: {:?}",
                &clipped_line.line.start.x, &player.position
            );
        }

        let sector = get_sector_from_vertex(map, &map_object.position);
        if sector.is_none() {
            // Shouldn't happen, but let's not panic if it does.
            println!("Thing is outside map: {:?}", map_object);
            continue;
        }

        let sector = sector.unwrap();

        let light_level = if map_object.state.full_bright {
            255
        } else {
            sector.borrow().light_level
        };

        let player_height = player.floor_height + PLAYER_EYE_HEIGHT;
        let z = sector.borrow().floor_height;
        let mut bottom_height = z as f32 - player_height;
        let mut top_height = z as f32 + picture.bitmap.height as f32 - 1.0 - player_height;

        // Add picture vertical offsets
        bottom_height += picture.top_offset as f32 - picture.bitmap.height as f32;
        top_height += picture.top_offset as f32 - picture.bitmap.height as f32;

        // Make bottom and top lines
        let bottom = make_sidedef_non_vertical_line(&clipped_line.line, bottom_height);
        let top = make_sidedef_non_vertical_line(&clipped_line.line, top_height);

        // top_seg_clip and bottom_seg_clip is the area not obscured.
        // It starts off all of the screen and gets reduced by the segs in front
        // of the map object.
        let mut top_seg_clip: [i16; SCREEN_WIDTH as usize] = [-1; SCREEN_WIDTH as usize];
        let mut bottom_seg_clip: [i16; SCREEN_WIDTH as usize] =
            [SCREEN_HEIGHT as i16; SCREEN_WIDTH as usize];

        // Loop over all segs and fill out the seg_clip arrays.
        for seg in &mut *segs {
            // Ignore segs behind the map object

            // The following logic can be found in R_DrawSprite in r_things.c.

            // Determine the x coordinate of the line closest (min) and furthest (max) from us
            let min_x = seg
                .clipped_line
                .line
                .start
                .x
                .min(seg.clipped_line.line.end.x);
            let max_x = seg
                .clipped_line
                .line
                .start
                .x
                .max(seg.clipped_line.line.end.x);

            // The entire line is behind the thing
            if min_x > view_port_vertex.x {
                continue;
            }

            // The line is either to the left or the right of the thing from our
            // point of view. If the thing is on the line's right, then the line is
            // behind it.
            if max_x > view_port_vertex.x
                && !view_port_vertex.is_left_of_line(&seg.clipped_line.line)
            {
                continue;
            }

            for column in &seg.columns {
                let x = column.x as usize;

                // Floors and ceilings simply follow the seg.extends_to* flags for
                // solid walls.
                if seg.state == BitmapRenderState::SolidSeg {
                    if seg.extends_to_bottom {
                        bottom_seg_clip[x] = bottom_seg_clip[x].min(column.clipped_top_y as i16);
                    }

                    if seg.extends_to_top {
                        top_seg_clip[x] = top_seg_clip[x].max(column.clipped_bottom_y as i16);
                    }
                } else if seg.state == BitmapRenderState::TwoSidedSeg {
                    // For portals, it's everything above and below the portal top &
                    // bottom.

                    // Clip the top unless there is no ceiling due to the sky hack.
                    // https://doomwiki.org/wiki/Sky_hack
                    if seg.draw_ceiling {
                        top_seg_clip[x] = top_seg_clip[x].max(column.top_y as i16);
                    }

                    bottom_seg_clip[x] = bottom_seg_clip[x].min(column.bottom_y as i16);
                }
            }
        }

        // Prepare the render object for the map object
        let mut bitmap_render = BitmapRender::new(
            BitmapRenderState::MapObject,
            Some(Rc::clone(&picture.bitmap)),
            light_level,
            clipped_line.clone(),
            bottom.start.x,
            bottom.end.x,
            bottom_height,
            top_height,
            0,
            0,
            false,
            false,
            false,
        );

        // Loop from the left x to the right x, calculating the y screen coordinates
        // for the bottom and top.
        let bottom_delta = (bottom.start.y as f32 - bottom.end.y as f32)
            / (bottom.start.x as f32 - bottom.end.x as f32);
        let top_delta =
            (top.start.y as f32 - top.end.y as f32) / (top.start.x as f32 - top.end.x as f32);

        // The end is one shorter to prevent texture wrap arounds
        for x in bottom.start.x as i16..bottom.end.x as i16 {
            // Calculate top and bottom of the line
            let bottom_y =
                (bottom.start.y as f32 + (x as f32 - bottom.start.x as f32) * bottom_delta) as i16;
            let top_y = (top.start.y as f32 + (x as f32 - top.start.x as f32) * top_delta) as i16;

            let mut clipped_top_y = top_y;
            let mut clipped_bottom_y = bottom_y;

            clipped_top_y = clipped_top_y.max(top_seg_clip[x as usize]);
            clipped_bottom_y = clipped_bottom_y.min(bottom_seg_clip[x as usize]);

            clipped_top_y = max(0, clipped_top_y);
            clipped_bottom_y = min(SCREEN_HEIGHT as i16 - 1, clipped_bottom_y);

            bitmap_render.add_column(x, clipped_top_y, clipped_bottom_y, bottom_y, top_y);
        }

        map_object_bitmap_renders.push(bitmap_render);
    }

    // Sort the map objects back to front
    map_object_bitmap_renders.sort();
    map_object_bitmap_renders.reverse();

    // Render the map objects + all two sided segs in between them.
    for map_object_bitmap_render in &mut map_object_bitmap_renders {
        // Render any two sided textures behind the map object
        for seg in &mut *segs {
            if seg > map_object_bitmap_render {
                seg.render(pixels, palette);
            }
        }

        // Render the map object
        map_object_bitmap_render.render(pixels, palette);
    }
}
