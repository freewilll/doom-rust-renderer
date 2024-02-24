mod linedefs;
mod map;
mod vertexes;
mod wad;

use crate::map::Map;
use crate::wad::WadFile;

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
    let file_data = read_file("doom1.wad");
    let wad_file = WadFile::new(&file_data);
    Map::new(&wad_file, "e1m1");
}
