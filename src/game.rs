use crate::map::Map;
use crate::things::{get_thing_by_type, ThingTypes};
use crate::vertexes::Vertex;

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
const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;
const MAP_BORDER: u32 = 20;

pub struct Player {
    pub position: Vertex,
    pub angle: f32,
}

pub struct Game {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    map: Map,
    player: Player,
    pressed_keys: HashSet<Keycode>,
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
        }
    }

    fn transform_vertex_to_point_for_map(&self, v: &Vertex) -> Point {
        let x_size: f32 = (self.map.bounding_box.right - self.map.bounding_box.left).into();
        let y_size: f32 = (self.map.bounding_box.bottom - self.map.bounding_box.top).into();

        let screen_width: f32 = (SCREEN_WIDTH - MAP_BORDER * 2) as f32;
        let screen_height: f32 = (SCREEN_HEIGHT - MAP_BORDER * 2) as f32;
        let map_border: f32 = MAP_BORDER as f32;

        let x =
            (map_border + (v.x - self.map.bounding_box.left) as f32 * screen_width / x_size) as i32;
        let y = (map_border + screen_height
            - 1.0
            - (v.y - self.map.bounding_box.top) as f32 * screen_height / y_size)
            as i32;
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
        let start_delta = Vertex::new(length as i16, length as i16).rotate(self.player.angle);
        let end_vertex = start_vertex + &start_delta;
        let start_point = self.transform_vertex_to_point_for_map(&start_vertex);
        let end_point = self.transform_vertex_to_point_for_map(&end_vertex);

        let arrow = Vertex::new(arrow_length as i16, arrow_length as i16);
        let right_arrow_vertex = &end_vertex + &arrow.rotate(self.player.angle - PI - PI / 4.0);
        let left_arrow_vertex = &end_vertex + &arrow.rotate(self.player.angle - PI + PI / 4.0);
        let right_arrow_point = self.transform_vertex_to_point_for_map(&right_arrow_vertex);
        let left_arrow_point = self.transform_vertex_to_point_for_map(&left_arrow_vertex);

        self.canvas.draw_line(start_point, end_point).unwrap();
        self.canvas.draw_line(right_arrow_point, end_point).unwrap();
        self.canvas.draw_line(left_arrow_point, end_point).unwrap();
    }

    fn process_down_keys(&mut self) {
        const ROTATE_ANGLE: f32 = PI / 128.0;
        const MOVE_LENGTH: i16 = (SCREEN_WIDTH as f32 / 256.0) as i16;

        let alt_down = self.pressed_keys.contains(&Keycode::LAlt)
            || self.pressed_keys.contains(&Keycode::RAlt);

        let shift_down = self.pressed_keys.contains(&Keycode::LShift)
            || self.pressed_keys.contains(&Keycode::RShift);

        let move_length = if shift_down {
            MOVE_LENGTH * 2
        } else {
            MOVE_LENGTH
        };

        let rotate_angle = if shift_down {
            ROTATE_ANGLE * 2.0
        } else {
            ROTATE_ANGLE
        };

        if !alt_down && self.pressed_keys.contains(&Keycode::Left) {
            self.player.angle += rotate_angle;
        }

        if alt_down && self.pressed_keys.contains(&Keycode::Left) {
            self.player.position.x -= move_length;
        }

        if !alt_down && self.pressed_keys.contains(&Keycode::Right) {
            self.player.angle -= rotate_angle;
        }

        if alt_down && self.pressed_keys.contains(&Keycode::Right) {
            self.player.position.x += move_length;
        }

        if self.pressed_keys.contains(&Keycode::Up) {
            let delta = Vertex::new(move_length, move_length).rotate(self.player.angle);
            self.player.position = &self.player.position + &delta;
        }

        if self.pressed_keys.contains(&Keycode::Down) {
            let delta = Vertex::new(move_length, move_length).rotate(self.player.angle);
            self.player.position = &self.player.position - &delta;
        }
    }

    pub fn main_loop(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            self.canvas.set_draw_color(Color::RGB(0, 0, 0));
            self.canvas.clear();
            self.draw_map_linedefs();
            self.draw_map_nodes();
            self.draw_map_player();
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
