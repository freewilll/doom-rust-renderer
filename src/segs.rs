use crate::linedefs::Linedef;
use crate::vertexes::Vertex;
use crate::wad::{MapLumpName, WadFile};
use std::rc::Rc;

#[derive(Debug)]
pub struct Seg {
    pub id: i16,
    pub start_vertex: Rc<Vertex>, // Start
    pub end_vertex: Rc<Vertex>,   // End
    pub angle: i16,               // Angle, full circle is -32768 to 32767.
    pub linedef: Rc<Linedef>,     // Corresponding linedef
    pub direction: bool,          // False (same as linedef) or True (opposite of linedef)
    pub offset: i16,              // distance along linedef to start of seg
}

pub fn load_segs(
    wad_file: &WadFile,
    vertexes: &Vec<Rc<Vertex>>,
    linedefs: &Vec<Rc<Linedef>>,
    map_name: &str,
) -> Vec<Rc<Seg>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Segs);
    let count = dir_entry.size as usize / 12; // A seg is 12 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 12;
        let seg = Seg {
            id: i as i16,
            start_vertex: Rc::clone(&vertexes[wad_file.read_i16(offset) as usize]),
            end_vertex: Rc::clone(&vertexes[wad_file.read_i16(offset + 2) as usize]),
            angle: wad_file.read_i16(offset + 4),
            linedef: Rc::clone(&linedefs[wad_file.read_i16(offset + 6) as usize]),
            direction: wad_file.read_i16(offset + 8) != 0,
            offset: wad_file.read_i16(offset + 10),
        };
        results.push(Rc::new(seg));
    }

    results
}
