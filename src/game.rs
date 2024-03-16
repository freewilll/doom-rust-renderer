use regex::Regex;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::TextureCreator;
use sdl2::video::Window;
use sdl2::Sdl;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::flats::Flats;
use crate::geometry::Line;
use crate::linedefs::Flags;
use crate::map::Map;
use crate::nodes::NodeChild;
use crate::palette::Palette;
use crate::pictures::Pictures;
use crate::renderer::{Pixels, Renderer};
use crate::textures::{Texture, Textures};
use crate::things::{get_thing_by_type, ThingTypes};
use crate::vertexes::Vertex;
use crate::wad::WadFile;

const TITLE: &str = "A doom renderer in Rust";
pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 768;
const MAP_BORDER: u32 = 20;

#[derive(Debug)]
pub struct Player {
    pub position: Vertex,
    pub floor_height: f32, // Set to the height of the sector the player is in
    pub angle: f32,
}

pub const AVG_TICKS_MAXSAMPLES: u32 = 16;

// Keep track of a rolling average of frame render times.
// A "tick" is actually a f32 time interval in seconds
struct AvgTicksCounter {
    index: usize,
    sum: f32,
    rolling_sum: f32,
    list: Vec<f32>, // A circular buffer of length AVG_TICKS_MAXSAMPLES
}

impl AvgTicksCounter {
    fn new() -> AvgTicksCounter {
        let mut list = vec![0.0; AVG_TICKS_MAXSAMPLES as usize];
        list.iter_mut().for_each(|x| *x = 0.0);
        AvgTicksCounter {
            index: 0,
            sum: 0.0,
            rolling_sum: 0.0,
            list: list,
        }
    }

    fn get_avg_ticks(&mut self, new_tick: f32) -> f32 {
        self.sum += self.list[self.index];
        self.rolling_sum -= self.list[self.index];
        self.rolling_sum += new_tick;
        self.list[self.index] = new_tick;

        self.index += 1;
        if self.index == AVG_TICKS_MAXSAMPLES as usize {
            self.index = 0;
        }

        return self.rolling_sum as f32 / AVG_TICKS_MAXSAMPLES as f32;
    }

    fn get_fps(&mut self, new_tick: f32) -> f32 {
        1.0 / self.get_avg_ticks(new_tick)
    }
}

#[allow(dead_code)]
pub struct Game {
    sdl_context: Sdl,
    pub canvas: Canvas<Window>,
    avg_ticks_counter: AvgTicksCounter,
    map: Map,
    pub palette: Palette,
    player: Player,
    pressed_keys: HashSet<Keycode>,
    viewing_map: bool,  // Toggle the 2D map
    turbo: f32,         // Percentage speed increase
    pictures: Pictures, // Pictures (aka patches)
    flats: Flats,       // Flats
    textures: Textures,
    sky_texture: Rc<Texture>,
    print_fps: bool, // Show frames per second
}

impl Game {
    pub fn new(wad_file: Rc<WadFile>, map_name: &str, turbo: i16, print_fps: bool) -> Game {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(TITLE, SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window
            .into_canvas()
            .software()
            .present_vsync()
            .build()
            .unwrap();

        let map = Map::new(&wad_file, map_name);

        let player1_start = get_thing_by_type(&map.things, ThingTypes::Player1Start);
        let player = Player {
            position: Vertex::new(player1_start.x, player1_start.y),
            angle: player1_start.angle,
            floor_height: 0.0,
        };

        let palette = Palette::new(&wad_file);
        let pictures = Pictures::new(&wad_file);
        let flats = Flats::new(&wad_file);
        let mut textures = Textures::new(&wad_file);

        let sky_texture = Self::get_sky_texture(map_name, &mut textures);

        let mut game = Game {
            sdl_context,
            canvas,
            avg_ticks_counter: AvgTicksCounter::new(),
            map,
            player,
            pressed_keys: HashSet::new(),
            viewing_map: false,
            turbo: (turbo as f32) / 100.0,
            palette,
            pictures,
            flats,
            textures,
            sky_texture: Rc::clone(&sky_texture),
            print_fps,
        };

        // Set initial player height
        game.update_current_player_height();

        game
    }

    // Determine which sky texture to be used based on the map name
    fn get_sky_texture(map_name: &str, textures: &mut Textures) -> Rc<Texture> {
        let doom1_re = Regex::new(r"e(?<episode>\d+)m(?<map>\d+)").unwrap();
        if let Some(caps) = doom1_re.captures(map_name) {
            let episode = caps["episode"].parse::<i32>().unwrap();

            return match episode {
                1 => Rc::clone(&textures.get("SKY1")),
                2 => Rc::clone(&textures.get("SKY2")),
                3 => Rc::clone(&textures.get("SKY3")),
                _ => Rc::clone(&textures.get("SKY1")), // Should not happen
            };
        }

        let doom2_re = Regex::new(r"(?<map>\d\d)").unwrap();
        if let Some(caps) = doom2_re.captures(map_name) {
            let map = caps["map"].parse::<i32>().unwrap();

            if map < 12 {
                return Rc::clone(&textures.get("SKY1"));
            } else if map < 21 {
                return Rc::clone(&textures.get("SKY2"));
            } else {
                return Rc::clone(&textures.get("SKY3"));
            }
        }

        // Fall back to something
        Rc::clone(&textures.get("SKY1"))
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
            if linedef.flags & Flags::DONTDRAW > 0 {
                continue;
            } else if linedef.flags & Flags::TWOSIDED > 0 {
                self.canvas.set_draw_color(Color::RGB(255, 255, 0));
            } else {
                self.canvas.set_draw_color(Color::RGB(255, 0, 0));
            }

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

    fn process_down_keys(&mut self, duration: &Duration) {
        let rotate_factor: f32 = duration.as_millis() as f32 * 0.0025; // radians/msec
        let move_factor: f32 = duration.as_millis() as f32 * 0.291; // 291 mu/sec

        let alt_down = self.pressed_keys.contains(&Keycode::LAlt)
            || self.pressed_keys.contains(&Keycode::RAlt);

        let shift_down = self.pressed_keys.contains(&Keycode::LShift)
            || self.pressed_keys.contains(&Keycode::RShift);

        let move_length = if shift_down {
            move_factor * self.turbo * 2.0
        } else {
            move_factor * self.turbo
        };

        let rotate_angle = if shift_down {
            rotate_factor * self.turbo * 2.0
        } else {
            rotate_factor * self.turbo
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
                            self.player.floor_height = sidedef.sector.floor_height as f32;
                            return;
                        };
                    }
                    return;
                }
            }
        }
    }

    pub fn main_loop(&mut self) {
        // Create the texture + pixels for the renderer
        let texture_creator: TextureCreator<_> = self.canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
        let mut pixels = Pixels::new();

        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            let t0 = Instant::now();

            if self.viewing_map {
                self.canvas.set_draw_color(Color::RGB(0, 0, 0));
                self.canvas.clear();

                self.draw_map_linedefs();
                self.draw_map_player();
            } else {
                pixels.clear();

                Renderer::new(
                    &mut pixels,
                    &self.map,
                    &mut self.textures,
                    Rc::clone(&self.sky_texture),
                    &mut self.flats,
                    &mut self.palette,
                    &self.player,
                    self.avg_ticks_counter.sum,
                )
                .render();

                texture
                    .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                        buffer.copy_from_slice(pixels.pixels.as_ref());
                    })
                    .unwrap();

                let screen_rect = Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
                self.canvas
                    .copy(&texture, screen_rect, screen_rect)
                    .unwrap();
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
                        self.viewing_map = !self.viewing_map;
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

            let elapsed = t0.elapsed();
            let fps = self.avg_ticks_counter.get_fps(elapsed.as_secs_f32());
            if self.print_fps {
                println!("FPS {}", fps);
            }

            self.process_down_keys(&elapsed);
        }
    }
}
