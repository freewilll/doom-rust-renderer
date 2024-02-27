use sdl2::pixels::Color;
use std::rc::Rc;

use crate::game::Game;
use crate::nodes::{Node, NodeChild};
use crate::subsectors::SubSector;
use crate::vertexes::Vertex;

const COLORS: &'static [Color] = &[
    Color::RGB(0, 0, 255),   // Blue
    Color::RGB(0, 255, 0),   // Green
    Color::RGB(0, 255, 255), // Aqua
    Color::RGB(255, 0, 0),   // Red
    Color::RGB(255, 0, 255), // Purple
    Color::RGB(255, 255, 0), // Yellow
];

fn render_subsector(game: &mut Game, subsector: &SubSector, subsector_counter: &mut i32) {
    *subsector_counter += 1;
    if *subsector_counter < 5 {
        game.canvas
            .set_draw_color(COLORS[*subsector_counter as usize % COLORS.len()]);

        for seg in &subsector.segs {
            let start = game.transform_vertex_to_point_for_map(&seg.start_vertex);
            let end = game.transform_vertex_to_point_for_map(&seg.end_vertex);
            game.canvas.draw_line(start, end).unwrap();
        }
    }
}

// Recurse through the BSP tree, drawing the subsector leaves
// The BSP algorithm guarantees that the subsectors are visited back to front.
fn render_node(game: &mut Game, node: &Rc<Node>, subsector_counter: &mut i32) {
    let v1 = Vertex::new(node.x, node.y);
    let v2 = &v1 + &Vertex::new(node.dx, node.dy);

    let is_left = game.player.position.is_left_of_line(&v1, &v2);

    let (front_child, back_child) = if is_left {
        (&node.left_child, &node.right_child)
    } else {
        (&node.right_child, &node.left_child)
    };

    match front_child {
        NodeChild::Node(node) => {
            render_node(game, &node, subsector_counter);
        }
        NodeChild::SubSector(subsector) => {
            render_subsector(game, &subsector, subsector_counter);
        }
    }

    // TODO: Use the bounding box and only recurse into the back of the split
    // if the player viewintersecrs with it.

    match back_child {
        NodeChild::Node(node) => {
            render_node(game, &node, subsector_counter);
        }
        NodeChild::SubSector(subsector) => {
            render_subsector(game, &subsector, subsector_counter);
        }
    }
}

pub fn render_map(game: &mut Game) {
    let mut subsector_counter: i32 = 0;
    let root_node = Rc::clone(&game.map.root_node);
    render_node(game, &root_node, &mut subsector_counter);
}
