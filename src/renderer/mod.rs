mod bitmap_render;
mod bsp;
mod clipped_line;
mod constants;
mod map_objects;
mod misc;
mod pixels;
mod sdl_line;
mod segs;
mod sidedef_visplanes;
mod visplanes;

use std::rc::Rc;

use crate::flats::Flats;
use crate::game::Player;
use crate::geometry::Line;
use crate::map::Map;
use crate::map_objects::MapObjects;
use crate::nodes::{Node, NodeChild};
use crate::palette::Palette;
use crate::sprites::Sprites;
use crate::subsectors::SubSector;
use crate::textures::{Texture, Textures};
use crate::vertexes::Vertex;

pub use bsp::get_sector_from_vertex;
use map_objects::draw_map_objects;
pub use pixels::Pixels;
use segs::Segs;
use visplanes::draw_visplane;

pub struct Renderer<'a> {
    segs: Segs<'a>,
    map: &'a Map,
    map_objects: &'a MapObjects,
    sprites: &'a mut Sprites,
    sky_texture: Rc<Texture>,
}

impl Renderer<'_> {
    pub fn new<'a>(
        pixels: &'a mut Pixels,
        map: &'a Map,
        map_objects: &'a MapObjects,
        textures: &'a mut Textures,
        sprites: &'a mut Sprites,
        sky_texture: Rc<Texture>,
        flats: &'a mut Flats,
        palette: &'a Palette,
        player: &'a Player,
        timestamp: f32,
    ) -> Renderer<'a> {
        let segs = Segs::new(pixels, textures, flats, palette, player, timestamp);

        Renderer {
            segs,
            map,
            map_objects,
            sprites,
            sky_texture,
        }
    }

    // Process all segs in a subsector
    fn process_subsector(&mut self, subsector: &SubSector) {
        for seg in &subsector.segs {
            self.segs.process_seg(seg);
        }
    }

    // Recurse through the BSP tree, drawing the subsector leaves
    // The BSP algorithm guarantees that the subsectors are visited front to back.
    fn render_node(&mut self, node: &Rc<Node>) {
        let v1 = Vertex::new(node.x, node.y);
        let v2 = &v1 + &Vertex::new(node.dx, node.dy);

        let is_left = self
            .segs
            .player
            .position
            .is_left_of_line(&Line::new(&v1, &v2));

        let (front_child, back_child) = if is_left {
            (&node.left_child, &node.right_child)
        } else {
            (&node.right_child, &node.left_child)
        };

        match front_child {
            NodeChild::Node(node) => {
                self.render_node(node);
            }
            NodeChild::SubSector(subsector) => {
                self.process_subsector(subsector);
            }
        }

        // TODO: Use the bounding box and only recurse into the back of the split
        // if the player view intersects with it.
        match back_child {
            NodeChild::Node(node) => {
                self.render_node(node);
            }
            NodeChild::SubSector(subsector) => {
                self.process_subsector(subsector);
            }
        }
    }

    fn draw_visplanes(&mut self) {
        for visplane in &self.segs.visplanes {
            draw_visplane(
                self.segs.pixels,
                self.segs.palette,
                self.segs.player,
                Rc::clone(&self.sky_texture),
                visplane,
            );
        }
    }

    pub fn render(&mut self) {
        let root_node = Rc::clone(&self.map.root_node);
        self.render_node(&root_node);

        self.draw_visplanes();

        self.segs.segs.reverse(); // Sort segs back to front
        draw_map_objects(
            &mut self.segs.segs,
            self.segs.pixels,
            self.map_objects,
            self.segs.player,
            self.sprites,
            self.map,
            self.segs.palette,
        );

        self.segs.draw_remaining_segs();
    }
}
