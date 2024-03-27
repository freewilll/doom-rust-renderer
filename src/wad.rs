use std::collections::HashMap;
use std::ffi::CStr;
use std::rc::Rc;
use std::{fmt, str};

// An enum which encodes the relative position in the wad file for map lumps
#[allow(dead_code)]
pub enum MapLumpName {
    Things = 1, // Monsters, weapons, keys, etc
    Linedefs,   // Lines
    Sidedefs,   // What's on the side of a line
    Vertexes,   // Vertexes that make up the lines
    Segs,       // Lines, split by the BSP builder
    Ssectors,   // Sectors, split by the BSP builder
    Nodes,      // BSP tree
    Sectors,    // Closed polygons made up of linedefs
    Reject,     // Precalculation if direct line of sight between sectors is possibl
    Blockmap,   // A grid of blocks used for collision detection
}

impl ToString for MapLumpName {
    fn to_string(&self) -> String {
        match self {
            MapLumpName::Things => "THINGS".to_string(),
            MapLumpName::Linedefs => "LINEDEFS".to_string(),
            MapLumpName::Sidedefs => "SIDEDEFS".to_string(),
            MapLumpName::Vertexes => "VERTEXES".to_string(),
            MapLumpName::Segs => "SEGS".to_string(),
            MapLumpName::Ssectors => "SSECTORS".to_string(),
            MapLumpName::Nodes => "NODES".to_string(),
            MapLumpName::Sectors => "SECTORS".to_string(),
            MapLumpName::Reject => "REJECT".to_string(),
            MapLumpName::Blockmap => "BLOCKMAP".to_string(),
        }
    }
}

// Wad file header
pub struct Header {
    magic: String,       // Magic 4-character string, IWAD or PWAD
    pub lump_count: u32, // Amount of lumps (files)
    dir_offset: u32,     // Offset  to the directory table
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Header: magic: {}, lump_count: {}, dir_offset: {}",
            self.magic, self.lump_count, self.dir_offset
        )
    }
}

// Read the WAD file header from file data
impl Header {
    pub fn read(file: &[u8]) -> Header {
        Header {
            magic: str::from_utf8(&file[0..4]).unwrap().to_string(),
            lump_count: u32::from_le_bytes(file[4..8].try_into().unwrap()),
            dir_offset: u32::from_le_bytes(file[8..12].try_into().unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct DirEntry {
    pub index: i16,   // Index in the lump list
    pub name: String, // Lump name
    pub offset: u32,  // Lump offset in file
    pub size: u32,    // Lump size
}

// A loaded WAD file
pub struct WadFile {
    pub file: Vec<u8>,
    pub header: Header,
    pub dirs_list: Vec<Rc<DirEntry>>,
    pub dirs_map: HashMap<String, Rc<DirEntry>>,
    pub first_sprite_lump: i16,
    pub last_sprite_lump: i16,
}

impl WadFile {
    // Load a WAD file
    pub fn new(file: Vec<u8>) -> WadFile {
        let header = Header::read(&file);

        // PWAD handling not implemented
        if header.magic != "IWAD" {
            panic!("Unhandled WAD file type: {}", header.magic);
        }

        let mut wad_file = WadFile {
            file: file,
            header: header,
            dirs_list: Vec::new(),
            dirs_map: HashMap::new(),
            first_sprite_lump: -1, // Populated later
            last_sprite_lump: -1,  // Populated later
        };

        wad_file.load_dirs();

        wad_file.first_sprite_lump = wad_file.get_dir_entry("S_START").unwrap().index;
        wad_file.last_sprite_lump = wad_file.get_dir_entry("S_END").unwrap().index;

        wad_file
    }

    // Read an 8 character null terminated string
    pub fn read_lump_name(&self, offset: usize) -> String {
        if self.file[offset + 7] == 0 {
            // The lump name is null terminated
            CStr::from_bytes_until_nul(&self.file[offset..offset + 8])
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        } else {
            // The lump name is exactly 8 bytes long
            str::from_utf8(&self.file[offset..offset + 8])
                .unwrap()
                .to_string()
        }
    }

    fn load_dirs(&mut self) {
        for i in 0..self.header.lump_count {
            // A directory entry is 16 bytes long
            let dir_entry_offset: usize = (self.header.dir_offset + i * 16).try_into().unwrap();

            let offset = u32::from_le_bytes(
                self.file[dir_entry_offset..dir_entry_offset + 4]
                    .try_into()
                    .unwrap(),
            );

            let size = u32::from_le_bytes(
                self.file[dir_entry_offset + 4..dir_entry_offset + 8]
                    .try_into()
                    .unwrap(),
            );

            let name = self.read_lump_name(dir_entry_offset + 8);

            let dir_entry = Rc::new(DirEntry {
                index: i as i16,
                name: name.clone().to_ascii_uppercase(),
                offset: offset,
                size: size,
            });
            self.dirs_map
                .insert(name.clone().to_ascii_uppercase(), Rc::clone(&dir_entry));
            self.dirs_list.push(dir_entry);
        }
    }

    #[allow(dead_code)]
    pub fn print_dirs(&self) {
        for dir in &self.dirs_list {
            println!("{:?}", dir);
        }
    }

    pub fn get_dir_entry(&self, name: &str) -> Result<&DirEntry, String> {
        if let Some(dir_entry) = self.dirs_map.get(&name.to_ascii_uppercase()) {
            return Ok(dir_entry);
        }

        Err(format!("Could not find lump {}", &name))
    }

    // Get lump for a map
    pub fn get_dir_entry_for_map_lump(&self, map_name: &str, lump_name: MapLumpName) -> &DirEntry {
        for (i, dir_entry) in self.dirs_list.iter().enumerate() {
            if dir_entry.name == map_name.to_ascii_uppercase() {
                return &self.dirs_list[i + lump_name as usize];
            }
        }

        panic!(
            "Could not find lump {} in map {}",
            lump_name.to_string(),
            map_name
        );
    }

    pub fn read_i16(&self, offset: usize) -> i16 {
        i16::from_le_bytes(self.file[offset..offset + 2].try_into().unwrap())
    }

    pub fn read_f32_from_i16(&self, offset: usize) -> f32 {
        i16::from_le_bytes(self.file[offset..offset + 2].try_into().unwrap()) as f32
    }

    pub fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(self.file[offset..offset + 4].try_into().unwrap())
    }
}
