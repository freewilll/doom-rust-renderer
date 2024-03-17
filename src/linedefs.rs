use crate::sidedefs::Sidedef;
use crate::vertexes::Vertex;
use crate::wad::{MapLumpName, WadFile};
use std::rc::Rc;

pub struct Flags;

#[allow(dead_code)]
impl Flags {
    pub const BLOCKING: i16 = 1; // Solid, is an obstacle.
    pub const BLOCKMONSTERS: i16 = 2; // // Blocks monsters only.
    pub const TWOSIDED: i16 = 4; // Backside will not be present at all if not two sided.
    pub const DONTPEGTOP: i16 = 8; // upper texture unpegged
    pub const DONTPEGBOTTOM: i16 = 16; // lower texture unpegged
    pub const SECRET: i16 = 32; // In AutoMap: don't map as two sided: IT'S A SECRET!
    pub const SOUNDBLOCK: i16 = 64; // Sound rendering: don't let sound cross two of these.
    pub const DONTDRAW: i16 = 128; // Don't draw on the automap at all.
    pub const MAPPED: i16 = 256; // Set if already seen, thus drawn in automap.
}

#[derive(Debug)]
pub struct Linedef {
    pub id: i16,
    pub start_vertex: Rc<Vertex>,
    pub end_vertex: Rc<Vertex>,
    pub flags: i16,
    pub special_type: i16,
    pub sector_tag: i16,
    pub front_sidedef: Option<Rc<Sidedef>>,
    pub back_sidedef: Option<Rc<Sidedef>>,
}

pub fn load_linedefs(
    wad_file: &WadFile,
    vertexes: &Vec<Rc<Vertex>>,
    sidedefs: &Vec<Rc<Sidedef>>,
    map_name: &str,
) -> Vec<Rc<Linedef>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Linedefs);
    let count = dir_entry.size as usize / 14; // A linedef is 14 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 14;

        let front_sidedef_index = wad_file.read_i16(offset + 10);
        let back_sidedef_index = wad_file.read_i16(offset + 12);

        let front_sidedef = if front_sidedef_index == -1 {
            None
        } else {
            Some(Rc::clone(&sidedefs[front_sidedef_index as usize]))
        };
        let back_sidedef = if back_sidedef_index == -1 {
            None
        } else {
            Some(Rc::clone(&sidedefs[back_sidedef_index as usize]))
        };

        let linedef = Linedef {
            id: i as i16,
            start_vertex: Rc::clone(&vertexes[wad_file.read_i16(offset) as usize]),
            end_vertex: Rc::clone(&vertexes[wad_file.read_i16(offset + 2) as usize]),
            flags: wad_file.read_i16(offset + 4),
            special_type: wad_file.read_i16(offset + 6),
            sector_tag: wad_file.read_i16(offset + 8),
            front_sidedef: front_sidedef,
            back_sidedef: back_sidedef,
        };
        results.push(Rc::new(linedef));
    }

    results
}
