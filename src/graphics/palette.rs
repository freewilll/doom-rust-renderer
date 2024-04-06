use crate::game::Game;
use crate::wad::WadFile;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

pub struct Palette {
    pub colors: [Color; 256], // Palette 0 in the PLAYPAL lump
}

impl Palette {
    pub fn new(wad_file: &WadFile) -> Palette {
        // Read the first palette, 768 bytes of 8-bit R, G, B values

        let playpal_dir_entry = wad_file.get_dir_entry("PLAYPAL").unwrap();
        let offset = playpal_dir_entry.offset as usize;

        let mut colors = [Color::RGB(0, 0, 0); 256];

        for (i, color) in colors.iter_mut().enumerate() {
            *color = Color::RGB(
                wad_file.file[offset + i * 3],
                wad_file.file[offset + i * 3 + 1],
                wad_file.file[offset + i * 3 + 2],
            );
        }

        Palette { colors }
    }
}

#[allow(dead_code)]
pub fn render_test(game: &mut Game) {
    for i in 0..16 {
        for j in 0..16 {
            let color = game.palette.colors[i * 16 + j];
            game.canvas.set_draw_color(color);
            let rect = Rect::new(i as i32 * 16, j as i32 * 16, 16, 16);
            game.canvas.fill_rect(rect).unwrap();
        }
    }
}
