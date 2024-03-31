use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub const PLAYER_EYE_HEIGHT: f32 = 41.0;

// The game ran on 320x200 but ended up on monitors with squarepixels and  320x240
// https://doomwiki.org/wiki/Aspect_ratio#:~:text=it%20was%20wide.-,Design%20of%20graphics,to%20this%20hardware%20video%20mode.
pub const ASPECT_RATIO_CORRECTION: f32 = 200.0 / 240.0;

// Do the perspetive transformation using a more broad screen then the
// actual screen. This is transformed back by the caller. The end result
// is everything being shown on the screen as it would have on the original
// VGA screens.
pub const GAME_SCREEN_WIDTH: f32 = SCREEN_WIDTH as f32 / ASPECT_RATIO_CORRECTION;
pub const GAME_CAMERA_FOCUS_X: f32 = GAME_SCREEN_WIDTH as f32 / 2.0 as f32;

pub const CAMERA_FOCUS_X: f32 = SCREEN_WIDTH as f32 / 2.0;
pub const CAMERA_FOCUS_Y: f32 = SCREEN_HEIGHT as f32 / 2.0;
