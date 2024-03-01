use crate::geometry::Line;
use crate::map::Map;
use crate::nodes::NodeChild;
use crate::renderer::render_map;
use crate::things::{get_thing_by_type, ThingTypes};
use crate::vertexes::Vertex;
use std::rc::Rc;

use std::collections::HashSet;
use std::f32::consts::PI;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

const TITLE: &str = "A doom renderer in Rust";
pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 768;
const MAP_BORDER: u32 = 20;

#[derive(Debug)]
pub struct Player {
    pub position: Vertex,
    pub angle: f32,
}

pub struct Game {
    sdl_context: Sdl,
    pub canvas: Canvas<Window>,
    pub map: Map,
    pub player: Player,
    pub pressed_keys: HashSet<Keycode>,
    pub viewing_map: i8,          // 0 = 3D, 1 = 3d + map, 2 = map
    pub player_floor_height: f32, // Set to the height of the sector the player is in
}

impl Game {
    pub fn new(map: Map) -> Game {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(TITLE, SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();

        let player1_start = get_thing_by_type(&map.things, ThingTypes::Player1Start);
        let player = Player {
            position: Vertex::new(player1_start.x, player1_start.y),
            angle: player1_start.angle,
        };

        Game {
            sdl_context: sdl_context,
            canvas: canvas,
            map: map,
            player: player,
            pressed_keys: HashSet::new(),
            viewing_map: 0,
            player_floor_height: 0.0,
        }
    }

    pub fn transform_vertex_to_point_for_map(&self, v: &Vertex) -> Point {
        let x_size: f32 = (self.map.bounding_box.right - self.map.bounding_box.left).into();
        let y_size: f32 = (self.map.bounding_box.bottom - self.map.bounding_box.top).into();

        let screen_width: f32 = (SCREEN_WIDTH - MAP_BORDER * 2) as f32;
        let screen_height: f32 = (SCREEN_HEIGHT - MAP_BORDER * 2) as f32;
        let map_border: f32 = MAP_BORDER as f32;

        let x = (map_border + (v.x - self.map.bounding_box.left) * screen_width / x_size) as i32;
        let y = (map_border + screen_height
            - 1.0
            - (v.y - self.map.bounding_box.top) * screen_height / y_size) as i32;
        Point::new(x.into(), y.into())
    }

    #[allow(dead_code)]
    fn draw_map_linedefs(&mut self) {
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));

        for linedef in &self.map.linedefs {
            let start_point = self.transform_vertex_to_point_for_map(&linedef.start_vertex);
            let end_point = self.transform_vertex_to_point_for_map(&linedef.end_vertex);
            self.canvas.draw_line(start_point, end_point).unwrap();
        }
    }

    #[allow(dead_code)]
    fn draw_map_nodes(&mut self) {
        self.canvas.set_draw_color(Color::RGB(255, 0, 0));

        for node in &self.map.nodes {
            let x = node.x;
            let y = node.y;
            let dx = node.dx;
            let dy = node.dy;

            let start_vertex = Vertex { x: x, y: y };
            let end_vertex = Vertex {
                x: x + dx,
                y: y + dy,
            };

            let start_point = self.transform_vertex_to_point_for_map(&start_vertex);
            let end_point = self.transform_vertex_to_point_for_map(&end_vertex);

            self.canvas.draw_line(start_point, end_point).unwrap();
        }
    }

    #[allow(dead_code)]
    fn draw_map_player(&mut self) {
        self.canvas.set_draw_color(Color::RGB(255, 255, 0));

        let length = SCREEN_WIDTH as f32 / 16.0;
        let arrow_length = SCREEN_WIDTH as f32 / 32.0;

        let start_vertex = &self.player.position;
        let start_delta = Vertex::new(length, 0.0).rotate(self.player.angle);
        let end_vertex = start_vertex + &start_delta;
        let start_point = self.transform_vertex_to_point_for_map(&start_vertex);
        let end_point = self.transform_vertex_to_point_for_map(&end_vertex);

        self.canvas.draw_line(start_point, end_point).unwrap();

        // Draw arrow lines
        let arrow = Vertex::new(arrow_length, 0.0);
        let right_arrow_vertex = &end_vertex + &arrow.rotate(self.player.angle - PI - PI / 4.0);
        let left_arrow_vertex = &end_vertex + &arrow.rotate(self.player.angle - PI + PI / 4.0);
        let right_arrow_point = self.transform_vertex_to_point_for_map(&right_arrow_vertex);
        let left_arrow_point = self.transform_vertex_to_point_for_map(&left_arrow_vertex);
        self.canvas.draw_line(right_arrow_point, end_point).unwrap();
        self.canvas.draw_line(left_arrow_point, end_point).unwrap();
    }

    fn process_down_keys(&mut self) {
        const ROTATE_ANGLE: f32 = PI / 128.0;
        const MOVE_LENGTH: f32 = SCREEN_WIDTH as f32 / 256.0;

        let alt_down = self.pressed_keys.contains(&Keycode::LAlt)
            || self.pressed_keys.contains(&Keycode::RAlt);

        let shift_down = self.pressed_keys.contains(&Keycode::LShift)
            || self.pressed_keys.contains(&Keycode::RShift);

        let move_length = if shift_down {
            MOVE_LENGTH * 2.0
        } else {
            MOVE_LENGTH
        };

        let rotate_angle = if shift_down {
            ROTATE_ANGLE * 2.0
        } else {
            ROTATE_ANGLE
        };

        // Rotation
        if !alt_down && self.pressed_keys.contains(&Keycode::Left) {
            self.player.angle += rotate_angle;
        }

        if !alt_down && self.pressed_keys.contains(&Keycode::Right) {
            self.player.angle -= rotate_angle;
        }

        // Strafe
        if alt_down && self.pressed_keys.contains(&Keycode::Left) {
            let delta = Vertex::new(move_length, 0.0).rotate(self.player.angle + PI / 2.0);
            self.player.position = &self.player.position + &delta;
            self.update_current_player_height();
        }

        if alt_down && self.pressed_keys.contains(&Keycode::Right) {
            let delta = Vertex::new(move_length, 0.0).rotate(self.player.angle + PI / 2.0);
            self.player.position = &self.player.position - &delta;
            self.update_current_player_height();
        }

        // Forward/backward
        if self.pressed_keys.contains(&Keycode::Up) {
            let delta = Vertex::new(move_length, 0.0).rotate(self.player.angle);
            self.player.position = &self.player.position + &delta;
            self.update_current_player_height();
        }

        if self.pressed_keys.contains(&Keycode::Down) {
            let delta = Vertex::new(move_length, 0.0).rotate(self.player.angle);
            self.player.position = &self.player.position - &delta;
            self.update_current_player_height();
        }
    }

    // Walk down the BSP tree to find the sector floor height the player is in
    fn update_current_player_height(&mut self) {
        let mut node = Rc::clone(&self.map.root_node);

        loop {
            let v1 = Vertex::new(node.x, node.y);
            let v2 = &v1 + &Vertex::new(node.dx, node.dy);

            let is_left = self.player.position.is_left_of_line(&Line::new(&v1, &v2));

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
                            self.player_floor_height = sidedef.sector.floor_height as f32;
                            return;
                        };
                    }
                    return;
                }
            }
        }
    }

    pub fn main_loop(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            self.canvas.set_draw_color(Color::RGB(0, 0, 0));
            self.canvas.clear();

            if self.viewing_map > 0 {
                self.draw_map_linedefs();
                self.draw_map_nodes();
                self.draw_map_player();
            }

            if self.viewing_map < 2 {
                render_map(self);
            }

            self.canvas.present();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => break 'running,

                    Event::KeyDown {
                        keycode: Some(Keycode::Tab),
                        ..
                    } => {
                        self.viewing_map = (self.viewing_map + 1) % 3;
                    }

                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        self.pressed_keys.insert(keycode);
                    }

                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => {
                        self.pressed_keys.remove(&keycode);
                    }

                    _ => {}
                }
            }

            self.process_down_keys();

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
