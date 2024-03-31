use std::rc::Rc;

use super::visplanes::Visplane;
use crate::flats::Flat;

// Keep track of the visplane state while processing a sidedef
pub struct SidedefVisPlanes {
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
    pub fn new(
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
    pub fn flush(&mut self, visplanes: &mut Vec<Visplane>) {
        if self.bottom_visplane_used {
            visplanes.push(self.bottom_visplane.clone());

            self.bottom_visplane =
                Visplane::new(&self.floor_flat, self.floor_height, self.light_level);
            self.bottom_visplane_used = false;
        }

        if self.top_visplane_used {
            visplanes.push(self.top_visplane.clone());

            self.top_visplane =
                Visplane::new(&self.ceiling_flat, self.ceiling_height, self.light_level);
            self.top_visplane_used = false;
        }
    }

    // Add a point to the bottom visplane
    pub fn add_bottom_point(&mut self, x: i16, top_y: i16, bottom_y: i16) {
        if !self.bottom_visplane_used {
            self.bottom_visplane.left = x;
        }

        self.bottom_visplane.right = x;

        self.bottom_visplane_used = true;
        self.bottom_visplane.top[x as usize] = top_y;
        self.bottom_visplane.bottom[x as usize] = bottom_y;
    }

    // Add a point to the top visplane
    pub fn add_top_point(&mut self, x: i16, top_y: i16, bottom_y: i16) {
        if !self.top_visplane_used {
            self.top_visplane.left = x;
        }

        self.top_visplane.right = x;

        self.top_visplane_used = true;
        self.top_visplane.top[x as usize] = top_y;
        self.top_visplane.bottom[x as usize] = bottom_y;
    }
}
