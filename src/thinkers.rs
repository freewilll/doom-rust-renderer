use std::rc::Rc;

use crate::lights::{FireFlicker, GlowingLight, LightFlash, StrobeFlash, FAST_DARK, SLOW_DARK};
use crate::map::Map;
use crate::map_objects::{MapObjectThinker, MapObjects};

pub trait Thinker {
    fn mutate(&mut self);
    fn kill(&mut self) {}
    fn explode(&mut self) {}
    fn respawn(&mut self) {}
}

fn init_sector_thinkers(thinkers: &mut Vec<Box<dyn Thinker>>, map: &Map) {
    for sector in &map.sectors {
        let special_type = sector.borrow().special_type;
        match special_type {
            1 => {
                // flickering lights
                thinkers.push(Box::new(LightFlash::new(map, Rc::clone(sector))))
            }
            2 => {
                // strobe fast
                thinkers.push(Box::new(StrobeFlash::new(
                    map,
                    Rc::clone(sector),
                    FAST_DARK,
                    false,
                )));
            }
            3 => {
                // strobe slow
                thinkers.push(Box::new(StrobeFlash::new(
                    map,
                    Rc::clone(sector),
                    SLOW_DARK,
                    false,
                )));
            }
            4 => {
                // strobe fast/death slime
                thinkers.push(Box::new(StrobeFlash::new(
                    map,
                    Rc::clone(sector),
                    FAST_DARK,
                    false,
                )));
                sector.borrow_mut().special_type = 4;
            }
            8 => {
                // glowing light
                thinkers.push(Box::new(GlowingLight::new(map, Rc::clone(sector))))
            }
            12 => {
                // sync strobe slow
                thinkers.push(Box::new(StrobeFlash::new(
                    map,
                    Rc::clone(sector),
                    SLOW_DARK,
                    true,
                )));
            }
            13 => {
                // sync strobe fast
                thinkers.push(Box::new(StrobeFlash::new(
                    map,
                    Rc::clone(sector),
                    FAST_DARK,
                    true,
                )));
            }
            17 => {
                // firelight flicker
                thinkers.push(Box::new(FireFlicker::new(map, Rc::clone(sector))))
            }

            _ => {}
        }
    }
}

fn init_map_obj_thinkers(thinkers: &mut Vec<Box<dyn Thinker>>, map_objects: &MapObjects) {
    for map_object in &map_objects.objects {
        thinkers.push(Box::new(MapObjectThinker::new(Rc::clone(map_object))));
    }
}

pub fn init_thinkers(thinkers: &mut Vec<Box<dyn Thinker>>, map: &Map, map_objects: &MapObjects) {
    init_sector_thinkers(thinkers, map);
    init_map_obj_thinkers(thinkers, map_objects);
}
