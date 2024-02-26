use crate::segs::Seg;
use crate::wad::{MapLumpName, WadFile};
use std::rc::Rc;

#[derive(Debug)]
#[allow(dead_code)]
pub struct SubSector {
    pub segs: Vec<Rc<Seg>>,
}

pub fn load_subsectors(
    wad_file: &WadFile,
    segs: &Vec<Rc<Seg>>,
    map_name: &str,
) -> Vec<Rc<SubSector>> {
    let dir_entry = wad_file.get_dir_entry_for_map_lump(map_name, MapLumpName::Ssectors);
    let count = dir_entry.size as usize / 4; // A subsector is 4 bytes long

    let mut results = Vec::new();
    for i in 0..count {
        let offset = dir_entry.offset as usize + i * 4;

        // Make vector of segs
        let seg_count = wad_file.read_i16(offset);
        let first_seg_number = wad_file.read_i16(offset + 2);
        let mut subsector_segs = Vec::new();
        for i in first_seg_number..first_seg_number + seg_count {
            subsector_segs.push(Rc::clone(&segs[i as usize]));
        }

        let subsector = SubSector {
            segs: subsector_segs,
        };
        results.push(Rc::new(subsector));
    }

    results
}
