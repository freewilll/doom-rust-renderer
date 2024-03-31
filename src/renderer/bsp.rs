use std::cell::RefCell;
use std::rc::Rc;

use crate::geometry::Line;
use crate::map::Map;
use crate::nodes::NodeChild;
use crate::sectors::Sector;
use crate::vertexes::Vertex;

// Walk the BSP tree to find the sector the vertex is in
// Returns None if the vertex is outside of the map.
pub fn get_sector_from_vertex(map: &Map, vertex: &Vertex) -> Option<Rc<RefCell<Sector>>> {
    let mut node = Rc::clone(&map.root_node);

    loop {
        let v1 = Vertex::new(node.x, node.y);
        let v2 = &v1 + &Vertex::new(node.dx, node.dy);

        let is_left = vertex.is_left_of_line(&Line::new(&v1, &v2));

        let child = if is_left {
            &node.left_child
        } else {
            &node.right_child
        };

        match child {
            NodeChild::Node(child_node) => node = Rc::clone(child_node),
            NodeChild::SubSector(subsector) => {
                for seg in &subsector.segs {
                    let linedef = &seg.linedef;

                    let opt_sidedef = if seg.direction {
                        &linedef.back_sidedef
                    } else {
                        &linedef.front_sidedef
                    };

                    if let Some(sidedef) = opt_sidedef {
                        return Some(sidedef.sector.clone());
                    };
                }
                return None;
            }
        }
    }
}
