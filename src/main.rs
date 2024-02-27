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

pub fn main() {
    let map_name = std::env::args().nth(1).unwrap_or("e1m1".to_string());

    let file_data = read_file("doom1.wad");
    let wad_file = WadFile::new(&file_data);
    let map = Map::new(&wad_file, map_name.as_str());

    let mut game = Game::new(map);
    game.main_loop();
}
