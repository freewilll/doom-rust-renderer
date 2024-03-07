use clap::{arg, command, Parser};
use std::rc::Rc;
use std::{fs::metadata, fs::File, io::Read};

mod game;
mod geometry;
mod linedefs;
mod map;
mod nodes;
mod palette;
mod pictures;
mod renderer;
mod sectors;
mod segs;
mod sidedefs;
mod subsectors;
mod textures;
mod things;
mod vertexes;
mod wad;

use game::Game;
use map::Map;
use wad::WadFile;

// Read a file into a u8 vector
fn read_file(filename: &str) -> Vec<u8> {
    let mut f = File::open(&filename).expect("Unable to open file");
    let metadata = metadata(&filename).expect("Unable to get metadata");
    let mut result = vec![0; metadata.len() as usize];
    f.read(&mut result).expect("Unable to read file");

    result
}

// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Map
    #[arg(short, long, default_value_t = String::from("e1m1") )]
    map: String,

    // Wad file
    #[arg(short, long, default_value_t = String::from("doom1.wad") )]
    wad: String,

    // Turbo
    #[arg(short, long, default_value_t = 100)]
    turbo: i16,

    // Print FPS
    #[arg(short, long, default_value_t = false)]
    print_fps: bool,
}

pub fn main() {
    let args = Args::parse();

    let file = read_file(&args.wad);
    let wad_file = Rc::new(WadFile::new(file));
    let map = Map::new(&wad_file, args.map.as_str());

    let mut game = Game::new(wad_file, map, args.turbo, args.print_fps);
    game.main_loop();
}
