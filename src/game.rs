use crate::map::Map;
use crate::vertexes::Vertex;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;
use std::time::Duration;

const TITLE: &str = "A doom renderer in Rust";
const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;
const MAP_BORDER: u32 = 20;

pub struct Game {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    map: Map,
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

        Game {
            sdl_context: sdl_context,
            canvas: canvas,
            map: map,
        }
    }

    fn transform_vertex_to_point_for_map(&self, v: &Vertex) -> Point {
        let x_size: f32 = (self.map.bottom_right.x - self.map.top_left.x).into();
        let y_size: f32 = (self.map.bottom_right.y - self.map.top_left.y).into();

        let screen_width: f32 = (SCREEN_WIDTH - MAP_BORDER * 2) as f32;
        let screen_height: f32 = (SCREEN_HEIGHT - MAP_BORDER * 2) as f32;
        let map_border: f32 = MAP_BORDER as f32;

        let x = (map_border + (v.x - self.map.top_left.x) as f32 * screen_width / x_size) as i32;
        let y = (map_border + screen_height
            - 1.0
            - (v.y - self.map.top_left.y) as f32 * screen_height / y_size) as i32;
        Point::new(x.into(), y.into())
    }

    #[allow(dead_code)]
    fn draw_map_linedefs(&mut self) {
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));

        for linedef in &self.map.linedefs {
            let start_vertex = &self.map.vertexes[linedef.start_vertex as usize];
            let end_vertex = &self.map.vertexes[linedef.end_vertex as usize];

            let start_point = self.transform_vertex_to_point_for_map(&start_vertex);
            let end_point = self.transform_vertex_to_point_for_map(&end_vertex);
            self.canvas.draw_line(start_point, end_point).unwrap();
        }
    }

    pub fn main_loop(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            self.canvas.set_draw_color(Color::RGB(0, 0, 0));
            self.canvas.clear();
            self.draw_map_linedefs();
            self.canvas.present();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
