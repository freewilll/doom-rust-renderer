use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::info::{MapObjectInfo, State, StateId, MAP_OBJECT_INFOS, STATES};
use crate::map::Map;
use crate::things::ThingTypes;
use crate::thinkers::Thinker;
use crate::vertexes::Vertex;

#[allow(dead_code)]
#[derive(Debug)]
pub struct MapObject {
    pub info: MapObjectInfo,
    pub state: State,
    pub position: Vertex,
    pub angle: f32, // In radians. 0=East, pi/2=North, pi=West, 3pi/2=South
    pub flags: i16,
}

#[derive(Debug)]
pub struct MapObjects {
    pub objects: Vec<Rc<RefCell<MapObject>>>,
}

impl MapObjects {
    pub fn new(map: &Map) -> MapObjects {
        let object_infos_map = Self::index_map_object_infos();

        let mut objects = Vec::new();

        for thing in &map.things {
            if (thing.thing_type >= ThingTypes::Player1Start as i16
                && thing.thing_type <= ThingTypes::Player4Start as i16)
                || thing.thing_type == ThingTypes::DeathMatchStart as i16
            {
                continue;
            }

            let map_object_info = object_infos_map.get(&thing.thing_type).unwrap();

            objects.push(Rc::new(RefCell::new(MapObject {
                info: map_object_info.clone(),
                state: STATES[map_object_info.spawn_state as usize].clone(),
                position: Vertex::new(thing.x, thing.y),
                angle: thing.angle,
                flags: thing.flags,
            })));
        }

        MapObjects { objects }
    }

    fn index_map_object_infos() -> HashMap<i16, MapObjectInfo> {
        let mut results: HashMap<i16, MapObjectInfo> = HashMap::new();
        for map_object_info in MAP_OBJECT_INFOS {
            results.insert(map_object_info.id, map_object_info);
        }

        results
    }
}

#[derive(Debug)]
pub struct MapObjectThinker {
    map_object: Rc<RefCell<MapObject>>,
    count: i16,
}

impl MapObjectThinker {
    pub fn new(map_object: Rc<RefCell<MapObject>>) -> MapObjectThinker {
        let count = map_object.borrow().state.tics;

        MapObjectThinker { map_object, count }
    }

    fn move_to_state(&mut self, state: StateId) {
        let next_state = STATES[state as usize].clone();
        let count = next_state.tics;
        let mut map_object = self.map_object.borrow_mut();
        map_object.state = next_state;
        self.count = count;
    }
}

impl Thinker for MapObjectThinker {
    fn mutate(&mut self) {
        if self.count == -1 {
            return;
        }

        self.count -= 1;
        if self.count > 0 {
            return;
        }

        let next_state = self.map_object.borrow().state.next_state;
        self.move_to_state(next_state);
    }

    fn kill(&mut self) {
        let death_state = self.map_object.borrow().info.death_state;
        if death_state != StateId::S_NULL {
            self.move_to_state(death_state);
        }
    }

    fn explode(&mut self) {
        let xdeath_state = self.map_object.borrow().info.xdeath_state;
        if xdeath_state != StateId::S_NULL {
            self.move_to_state(xdeath_state);
            return;
        }

        // Fall back to death state if there is no xdeath one
        self.kill();
    }

    fn respawn(&mut self) {
        let spawn_state = self.map_object.borrow().info.spawn_state;
        self.move_to_state(spawn_state);
    }
}

pub fn kill_everything(thinkers: &mut Vec<Box<dyn Thinker>>) {
    for thinker in thinkers {
        thinker.kill();
    }
}

pub fn explode_everything(thinkers: &mut Vec<Box<dyn Thinker>>) {
    for thinker in thinkers {
        thinker.explode();
    }
}

pub fn respawn_everything(thinkers: &mut Vec<Box<dyn Thinker>>) {
    for thinker in thinkers {
        thinker.respawn();
    }
}
