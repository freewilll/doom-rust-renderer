use std::cmp::{max, min};
use std::rc::Rc;

use crate::flats::{Flat, Flats};
use crate::game::Player;
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::geometry::Line;
use crate::linedefs::Flags;
use crate::palette::Palette;
use crate::segs::Seg;
use crate::sidedefs::Sidedef;
use crate::textures::Textures;

use super::bitmap_render::{render_vertical_bitmap_line, BitmapRender, BitmapRenderState};
use super::clipped_line::ClippedLine;
use super::constants::PLAYER_EYE_HEIGHT;
use super::misc::{clip_to_viewport, make_sidedef_non_vertical_line};
use super::pixels::Pixels;
use super::sdl_line::SdlLine;
use super::sidedef_visplanes::SidedefVisPlanes;
use super::visplanes::Visplane;

// The heart of the renderer. Process all walls & portals. Solid walls are rendered,
// portals are left to be rendered later with the map objects (things). A list of
// visplanes are created for the next stage of rendering.
pub struct Segs<'a> {
    // Game state
    pub pixels: &'a mut Pixels,
    pub palette: &'a mut Palette,
    pub player: &'a Player,
    textures: &'a mut Textures,
    flats: &'a mut Flats,
    timestamp: f32,

    // Outputs
    pub segs: Vec<BitmapRender>,  // Segs, rendered and unrendered
    pub visplanes: Vec<Visplane>, // Resulting visplanes

    // Internals
    hor_ocl: [bool; SCREEN_WIDTH as usize], // Horizontal occlusions
    floor_ver_ocl: [i16; SCREEN_WIDTH as usize], // Vertical occlusions for the floor
    ceiling_ver_ocl: [i16; SCREEN_WIDTH as usize], // Vertical occlusions for the ceiling
}

impl Segs<'_> {
    pub fn new<'a>(
        pixels: &'a mut Pixels,
        textures: &'a mut Textures,
        flats: &'a mut Flats,
        palette: &'a mut Palette,
        player: &'a Player,
        timestamp: f32,
    ) -> Segs<'a> {
        Segs {
            pixels,
            palette,
            player,
            textures,
            flats,
            timestamp,
            segs: Vec::new(),
            visplanes: Vec::new(),
            hor_ocl: [false; SCREEN_WIDTH as usize],
            floor_ver_ocl: [SCREEN_HEIGHT as i16; SCREEN_WIDTH as usize],
            ceiling_ver_ocl: [-1; SCREEN_WIDTH as usize],
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

    // Process a part of a sidedef.
    // This may involve drawing it, but might also involve processing occlusions and visplanes.
    fn process_sidedef(
        &mut self,
        clipped_line: &ClippedLine, // The clipped line in viewport coords
        sidedef: Rc<Sidedef>,       // The sidedef
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
        only_occlusions: bool,      // Don't draw, only add visplanes + occlusions
        is_lower_wall: bool,        // For portals: the rendered piece of wall
        is_upper_wall: bool,        // For portals: the rendered piece of wall
        draw_ceiling: bool,         // Set to false in a special case for sky texture
        is_two_sided_middle_wall: bool, // Two sided middle texture, add to list to draw later, don't add occlusions
    ) {
        let bottom = make_sidedef_non_vertical_line(&clipped_line.line, bottom_height);
        let top = make_sidedef_non_vertical_line(&clipped_line.line, top_height);

        let texture = if texture_name != "-" {
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

        let mut sidedef_visplanes = SidedefVisPlanes::new(
            light_level,
            floor_flat,
            ceiling_flat,
            floor_height,
            ceiling_height,
        );

        // Does the wall from from floor to ceiling?
        let is_full_height_wall = !is_lower_wall && !is_upper_wall && !only_occlusions;

        let bitmap_render_state = if is_two_sided_middle_wall {
            BitmapRenderState::TwoSidedSeg
        } else {
            BitmapRenderState::SolidSeg
        };

        let bitmap = texture
            .as_ref()
            .map_or_else(|| None, |t| Some(Rc::clone(&t.bitmap)));

        let mut bitmap_render = BitmapRender::new(
            bitmap_render_state,
            bitmap,
            light_level,
            clipped_line.clone(),
            bottom.start.x,
            bottom.end.x,
            bottom_height,
            top_height,
            sidedef.x_offset as i16 + seg_offset,
            sidedef.y_offset as i16 + offset_y as i16,
            is_lower_wall || (!is_two_sided_middle_wall && is_full_height_wall),
            is_upper_wall || (!is_two_sided_middle_wall && is_full_height_wall),
        );

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
                if in_ver_clipped_area {
                    if !is_two_sided_middle_wall && !only_occlusions {
                        if let Some(texture) = &texture {
                            render_vertical_bitmap_line(
                                // Wall/portal details
                                &mut self.pixels,
                                &self.palette,
                                &texture.bitmap,
                                light_level,
                                &clipped_line,
                                bottom.start.x,
                                bottom.end.x,
                                bottom_height,
                                top_height,
                                sidedef.x_offset as i16 + seg_offset,
                                sidedef.y_offset as i16 + offset_y as i16,
                                // Column details
                                x.into(),
                                clipped_top_y.into(),
                                clipped_bottom_y.into(),
                                bottom_y.into(),
                                top_y.into(),
                            );
                        }
                    }

                    bitmap_render.add_column(x, clipped_top_y, clipped_bottom_y, bottom_y, top_y);
                }

                if !is_two_sided_middle_wall
                    && in_ver_clipped_area
                    && (is_full_height_wall || only_occlusions)
                {
                    let mut visplane_added = false;

                    // Process bottom visplane
                    if clipped_bottom_y < floor_ver_ocl {
                        if clipped_bottom_y != SCREEN_HEIGHT as i16 - 1 {
                            sidedef_visplanes.add_bottom_point(x, clipped_bottom_y, floor_ver_ocl);
                            visplane_added = true;
                        }
                    }

                    // Process top visplane
                    if !is_two_sided_middle_wall && draw_ceiling && clipped_top_y > ceiling_ver_ocl
                    {
                        if clipped_top_y != -1 {
                            if draw_ceiling {
                                sidedef_visplanes.add_top_point(x, ceiling_ver_ocl, clipped_top_y);
                            }
                            visplane_added = true;
                        }
                    }

                    if !visplane_added {
                        // Line is occluded, flush visplanes
                        sidedef_visplanes.flush(&mut self.visplanes);
                    }
                } else if !is_two_sided_middle_wall
                    && !in_ver_clipped_area
                    && (is_full_height_wall || only_occlusions)
                    && floor_ver_ocl > ceiling_ver_ocl
                {
                    // The sidedef is occluded. However, there is still is a vertical
                    // unoccluded gap. Fill it with the floor/ceiling texture belonging to
                    // the sidedef. This is rare, but happens e.g. in doom1 e1m1 when in
                    // the hidden ahrea going down the stairs to the outside area.

                    if bottom_y <= ceiling_ver_ocl {
                        sidedef_visplanes.add_bottom_point(x, ceiling_ver_ocl, floor_ver_ocl);

                        // Occlude the entire vertical line
                        self.occlude_vertical_line(x);
                    }

                    if draw_ceiling && top_y >= floor_ver_ocl {
                        if draw_ceiling {
                            sidedef_visplanes.add_top_point(x, ceiling_ver_ocl, floor_ver_ocl);
                        }

                        // Occlude the entire vertical line
                        self.occlude_vertical_line(x);
                    }
                }

                if !is_two_sided_middle_wall && in_ver_clipped_area && only_occlusions {
                    self.floor_ver_ocl[x as usize] = clipped_bottom_y;

                    if draw_ceiling {
                        self.ceiling_ver_ocl[x as usize] = clipped_top_y;
                    }
                }

                // Update vertical occlusions
                if !is_two_sided_middle_wall && in_ver_clipped_area && is_lower_wall {
                    self.floor_ver_ocl[x as usize] = clipped_top_y;
                }

                if !is_two_sided_middle_wall && in_ver_clipped_area && is_upper_wall {
                    self.ceiling_ver_ocl[x as usize] = clipped_bottom_y;
                }
            } else {
                // Line is occluded, flush visplanes
                sidedef_visplanes.flush(&mut self.visplanes);
            }

            if !is_two_sided_middle_wall && is_full_height_wall {
                // A vertical line occludes everything behind it
                self.occlude_vertical_line(x);
            }
        }

        sidedef_visplanes.flush(&mut self.visplanes);

        self.segs.push(bitmap_render);
    }

    // Process a seg
    pub fn process_seg(&mut self, seg: &Seg) {
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

        let front_sector = &front_sidedef.sector.borrow();

        // Get the floor and ceiling height from the front sector
        let floor_height = front_sector.floor_height as f32;
        let mut ceiling_height = front_sector.ceiling_height as f32;

        // For portals, get the bottom and top heights by looking at the back
        // sector.
        let (opt_portal_bottom_height, mut opt_portal_top_height) = match opt_back_sidedef {
            Some(back_sidedef) => {
                let back_sector = &back_sidedef.sector;

                let opt_portal_bottom_height =
                    if back_sector.borrow().floor_height > front_sector.floor_height {
                        Some(back_sector.borrow().floor_height as f32)
                    } else {
                        None
                    };

                let opt_portal_top_height =
                    if back_sector.borrow().ceiling_height < front_sector.ceiling_height {
                        Some(back_sector.borrow().ceiling_height as f32)
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
        let player_height = &self.player.floor_height + PLAYER_EYE_HEIGHT;

        // Check one line to ensure we're not facing the back of it
        let floor =
            make_sidedef_non_vertical_line(&clipped_line.line, floor_height - player_height);

        // We are facing the non-rendered side of the segment.
        if floor.start.x > floor.end.x {
            return;
        }

        let floor_flat = self
            .flats
            .get_animated(front_sector.floor_texture.as_str(), self.timestamp);
        let ceiling_flat = self
            .flats
            .get_animated(front_sector.ceiling_texture.as_str(), self.timestamp);

        let mut draw_ceiling = true;

        // If both the front and back sector are sky, then don't draw the top linedef
        // and don't draw the sky.
        // https://doomwiki.org/wiki/Sky_hack
        // This follows the gory details in r_segs.c
        if let Some(back_sidedef) = opt_back_sidedef {
            if front_sidedef
                .sector
                .borrow()
                .ceiling_texture
                .contains("SKY")
                && back_sidedef.sector.borrow().ceiling_texture.contains("SKY")
            {
                opt_portal_top_height = None;
                ceiling_height = back_sidedef.sector.borrow().ceiling_height as f32;
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
                Rc::clone(&front_sidedef),
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
                false,
            );
        } else {
            // Process a portal

            // Process the portal's full height, only occlusions + visplanes are added
            self.process_sidedef(
                &clipped_line,
                Rc::clone(&front_sidedef),
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
                true, // Only add occlusions/visplanes
                false,
                false,
                draw_ceiling,
                false,
            );

            // Process the middle bit, adding it to the list of two sided
            // textures to be drawn later together with the things.
            // Occlusions + visplanes are already dealt with.
            let mut mid_texture_floor_height = floor_height;
            let mut mid_texture_ceiling_height = ceiling_height;

            if let Some(portal_bottom_height) = opt_portal_bottom_height {
                mid_texture_floor_height = portal_bottom_height;
            }

            if let Some(portal_top_height) = opt_portal_top_height {
                mid_texture_ceiling_height = portal_top_height;
            }

            self.process_sidedef(
                &clipped_line,
                Rc::clone(&front_sidedef),
                mid_texture_floor_height - player_height,
                mid_texture_ceiling_height - player_height,
                seg.offset,
                0,
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
                true, // is_two_sided_middle_wall
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
                    Rc::clone(&front_sidedef),
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
                    false,
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
                    Rc::clone(&front_sidedef),
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
                    false,
                );
            }
        }
    }

    // Draw remaining two sided segs
    pub fn draw_remaining_segs(&mut self) {
        for seg in &mut self.segs {
            seg.render(&mut self.pixels, &self.palette);
        }
    }
}
