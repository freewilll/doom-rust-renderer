use clap::{arg, command, Parser};

mod game;
mod geometry;
mod linedefs;
mod map;
mod nodes;
mod renderer;
mod sectors;
mod segs;
mod sidedefs;
mod subsectors;
mod things;
mod vertexes;
mod wad;

use game::Game;
use map::Map;
use wad::WadFile;

use std::{fs::metadata, fs::File, io::Read};

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
}

pub fn main() {
    let args = Args::parse();

    let file_data = read_file(&args.wad);
    let wad_file = WadFile::new(&file_data);
    let map = Map::new(&wad_file, args.map.as_str());

    let mut game = Game::new(map);
    game.main_loop();
}
