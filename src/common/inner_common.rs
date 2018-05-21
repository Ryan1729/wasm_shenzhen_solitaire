//in pixels
pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
pub const SCREEN_LENGTH: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const PALETTE: [u32; 16] = [
    0xff000000,
    0xff532b1d,
    0xff53257e,
    0xff518700,
    0xff3652ab,
    0xff4f575f,
    0xffc7c3c2,
    0xffe8f1ff,
    0xff4d00ff,
    0xff00a3ff,
    0xff27ecff,
    0xff36e400,
    0xffffad29,
    0xff9c7683,
    0xffa877ff,
    0xffaaccff,
];

pub struct GameState {}
