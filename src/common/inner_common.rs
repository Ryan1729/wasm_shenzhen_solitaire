//in pixels
pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
pub const SCREEN_LENGTH: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

// reportedly colourblind friendly colours
// https://twitter.com/ea_accessible/status/968595073184092160
pub mod colours {
    pub const BLUE: u32 = 0xFFE15233;
    pub const GREEN: u32 = 0xFF6EB030;
    pub const RED: u32 = 0xFF4949DE;
    pub const YELLOW: u32 = 0xFF37B9FF;
    pub const PURPLE: u32 = 0xFF543353;
    pub const GREY: u32 = 0xFF8B7D5A;
    pub const GRAY: u32 = GREY;
    pub const WHITE: u32 = 0xFFEEEEEE;
    pub const BLACK: u32 = 0xFF222222;
}
pub use colours::*;

pub struct GameState {}
