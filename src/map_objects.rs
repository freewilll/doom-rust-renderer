use std::collections::HashMap;

use crate::info::{MapObjectInfo, State, MAP_OBJECT_INFOS, STATES};
use crate::map::Map;
use crate::things::ThingTypes;
use crate::vertexes::Vertex;

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
    pub objects: Vec<MapObject>,
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

            objects.push(MapObject {
                info: map_object_info.clone(),
                state: STATES[map_object_info.spawn_state as usize].clone(),
                position: Vertex::new(thing.x, thing.y),
                angle: thing.angle,
                flags: thing.flags,
            });
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
