use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

use crate::map::{Map, Sector};
use crate::thinkers::Thinker;

pub const SLOW_DARK: i16 = 35;
pub const FAST_DARK: i16 = 15;

const STROBE_BRIGHT: i16 = 5;
const GLOW_SPEED: i16 = 8;

// Find the minimum light level of all neighboring sectors
fn find_min_surrounding_light(map: &Map, sector_id: i16, max: i16) -> i16 {
    let mut light_level = max;

    for linedef in &map.linedefs {
        if let Some(front_sidedef) = &linedef.front_sidedef {
            let sector = front_sidedef.sector.borrow();
            if sector.id == sector_id {
                if let Some(back_sidedef) = &linedef.back_sidedef {
                    let sector = back_sidedef.sector.borrow();
                    light_level = light_level.min(sector.light_level)
                }
            }
        }

        if let Some(back_sidedef) = &linedef.back_sidedef {
            let sector = back_sidedef.sector.borrow();
            if sector.id == sector_id {
                if let Some(front_sidedef) = &linedef.front_sidedef {
                    let sector = front_sidedef.sector.borrow();
                    light_level = light_level.min(sector.light_level)
                }
            }
        }
    }

    light_level
}

// See p_lights.c
// The light flickers between a max and low light with different random intervals
#[derive(Debug)]
pub struct LightFlash {
    sector: Rc<RefCell<Sector>>, // The sector to change the light on
    rng: ThreadRng,              // Random number generator
    min_light: i16,              // Minimum light level
    max_light: i16,              // Maximum light level
    min_time: i16,               // Random interval when at the minimum light level
    max_time: i16,               // Random interval when at the maximum light level
    count: i16,                  // Counts down when at min or max
}

impl LightFlash {
    pub fn new(map: &Map, sector: Rc<RefCell<Sector>>) -> LightFlash {
        let min_light =
            find_min_surrounding_light(map, sector.borrow().id, sector.borrow().light_level);
        let max_light = sector.borrow().light_level;
        let min_time = 7;
        let max_time = 64;
        let mut rng = rand::thread_rng();

        LightFlash {
            sector,
            min_light,
            max_light,
            min_time,
            max_time,
            count: rng.gen_range(1..max_time + 1),
            rng,
        }
    }
}

impl Thinker for LightFlash {
    fn mutate(&mut self) {
        let mut sector = self.sector.borrow_mut();

        self.count -= 1;
        if self.count > 0 {
            return;
        }

        if sector.light_level == self.max_light {
            // Go dark & wait random(min_time)

            sector.light_level = self.min_light;
            self.count = self.rng.gen_range(1..self.min_time + 1);
        } else {
            // Go light & wait random(max_time)

            sector.light_level = self.max_light;
            self.count = self.rng.gen_range(1..self.max_time + 1);
        }
    }
}

// See p_lights.c
// The light goes on and off with fixed intervals
#[derive(Debug)]
pub struct StrobeFlash {
    sector: Rc<RefCell<Sector>>, // The sector to change the light on
    min_light: i16,              // Minimum light level
    max_light: i16,              // Maximum light level
    dark_time: i16,              // Time spent in dark light
    bright_time: i16,            // Time spent in bright light
    count: i16,                  // Counts down when at min or max
}

impl StrobeFlash {
    pub fn new(
        map: &Map,
        sector: Rc<RefCell<Sector>>,
        dark_time: i16,
        in_sync: bool,
    ) -> StrobeFlash {
        let mut min_light =
            find_min_surrounding_light(map, sector.borrow().id, sector.borrow().light_level);

        let max_light = sector.borrow().light_level;

        if min_light == max_light {
            min_light = 0;
        }

        let mut rng = rand::thread_rng();
        let count = if in_sync { 1 } else { rng.gen_range(1..9) };

        StrobeFlash {
            sector,
            min_light,
            max_light,
            dark_time,
            bright_time: STROBE_BRIGHT,
            count,
        }
    }
}

impl Thinker for StrobeFlash {
    fn mutate(&mut self) {
        let mut sector = self.sector.borrow_mut();

        self.count -= 1;
        if self.count > 0 {
            return;
        }

        if sector.light_level == self.max_light {
            // Go dark & wait dark_time

            sector.light_level = self.min_light;
            self.count = self.dark_time;
        } else {
            // Go light & wait bright_time

            sector.light_level = self.max_light;
            self.count = self.bright_time;
        }
    }
}

// See p_lights.c
// Slowly go back and forward between min and max light
#[derive(Debug)]
pub struct GlowingLight {
    sector: Rc<RefCell<Sector>>, // The sector to change the light on
    min_light: i16,              // Minimum light level
    max_light: i16,              // Maximum light level
    going_up: bool,              // Going up or down
}

impl GlowingLight {
    pub fn new(map: &Map, sector: Rc<RefCell<Sector>>) -> GlowingLight {
        let min_light =
            find_min_surrounding_light(map, sector.borrow().id, sector.borrow().light_level);
        let max_light = sector.borrow().light_level;

        GlowingLight {
            sector,
            min_light,
            max_light,
            going_up: false,
        }
    }
}

impl Thinker for GlowingLight {
    fn mutate(&mut self) {
        let mut sector = self.sector.borrow_mut();

        if self.going_up {
            sector.light_level += GLOW_SPEED;

            if sector.light_level >= self.max_light {
                sector.light_level -= GLOW_SPEED;
                self.going_up = false;
            }
        } else {
            sector.light_level -= GLOW_SPEED;

            if sector.light_level <= self.min_light {
                sector.light_level += GLOW_SPEED;
                self.going_up = true;
            }
        }
    }
}

// See p_lights.c
// Spike to maxlight then randomly go down to minlight, then spike back up again.
#[derive(Debug)]
pub struct FireFlicker {
    sector: Rc<RefCell<Sector>>, // The sector to change the light on
    rng: ThreadRng,              // Random number generator
    min_light: i16,              // Minimum light level
    max_light: i16,              // Maximum light level
    count: i16,
}

impl FireFlicker {
    pub fn new(map: &Map, sector: Rc<RefCell<Sector>>) -> FireFlicker {
        let min_light =
            find_min_surrounding_light(map, sector.borrow().id, sector.borrow().light_level) + 16;
        let max_light = sector.borrow().light_level;

        FireFlicker {
            sector,
            rng: rand::thread_rng(),
            min_light,
            max_light,
            count: 4,
        }
    }
}

impl Thinker for FireFlicker {
    fn mutate(&mut self) {
        let mut sector = self.sector.borrow_mut();

        self.count -= 1;
        if self.count > 0 {
            return;
        }

        let amount = self.rng.gen_range(0..4) * 16;

        if sector.light_level - amount < self.min_light {
            sector.light_level = self.min_light;
        } else {
            sector.light_level = self.max_light - amount;
        }

        self.count = 4;
    }
}
