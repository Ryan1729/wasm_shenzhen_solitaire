use inner_common::*;

pub struct Framebuffer {
    pub buffer: Vec<u32>,
}

impl PartialEq for Framebuffer {
    fn eq(&self, other: &Framebuffer) -> bool {
        &self.buffer[..] == &other.buffer[..]
    }
}

impl Eq for Framebuffer {}

macro_rules! red {
    ($colour:expr) => {
        $colour & 0xFF
    };
}

macro_rules! green {
    ($colour:expr) => {
        ($colour & 0xFF_00) >> 8
    };
}

macro_rules! blue {
    ($colour:expr) => {
        ($colour & 0xFF_00_00) >> 16
    };
}

macro_rules! alpha {
    ($colour:expr) => {
        ($colour & 0xFF_00_00_00) >> 24
    };
}

macro_rules! colour {
    ($red:expr, $green:expr, $blue:expr, $alpha:expr) => {
        $red | $green << 8 | $blue << 16 | $alpha << 24
    };
}

macro_rules! set_alpha {
    ($colour:expr, $alpha:expr) => {
        ($colour & 0x00_FF_FF_FF) | $alpha << 24
    };
}

#[allow(dead_code)]
impl Framebuffer {
    pub fn new() -> Framebuffer {
        Framebuffer::default()
    }

    pub fn xy_to_i(x: usize, y: usize) -> usize {
        y.saturating_mul(SCREEN_WIDTH).saturating_add(x)
    }

    pub fn draw_filled_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        colour: u32,
    ) {
        let one_past_right_edge = x + width;
        let one_past_bottom_edge = y + height;

        for current_y in y..one_past_bottom_edge {
            for current_x in x..one_past_right_edge {
                let i = Framebuffer::xy_to_i(current_x, current_y);
                if i < self.buffer.len() {
                    self.buffer[i] = colour;
                }
            }
        }
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, colour: u32) {
        let one_past_right_edge = x + width;
        let one_past_bottom_edge = y + height;

        for current_y in y..one_past_bottom_edge {
            {
                let i = Framebuffer::xy_to_i(x, current_y);
                if i < self.buffer.len() {
                    self.buffer[i] = colour;
                }
            }

            {
                let i = Framebuffer::xy_to_i(one_past_right_edge - 1, current_y);
                if i < self.buffer.len() {
                    self.buffer[i] = colour;
                }
            }
        }

        for current_x in x..one_past_right_edge {
            {
                let i = Framebuffer::xy_to_i(current_x, y);
                if i < self.buffer.len() {
                    self.buffer[i] = colour;
                }
            }

            {
                let i = Framebuffer::xy_to_i(current_x, one_past_bottom_edge - 1);
                if i < self.buffer.len() {
                    self.buffer[i] = colour;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.buffer.len() {
            self.buffer[i] = 0;
        }
    }

    pub fn clearTo(&mut self, colour: u32) {
        for i in 0..self.buffer.len() {
            self.buffer[i] = colour;
        }
    }

    //see http://members.chello.at/~easyfilter/bresenham.html
    pub fn draw_crisp_circle(&mut self, xMid: usize, yMid: usize, radius: usize, colour: u32) {
        if xMid < radius || yMid < radius {
            if cfg!(debug_assertions) {
                console!(log, "draw_crisp_circle xMid < radius || yMid < radius");
            }

            return;
        }
        let mut r = radius as isize;
        let mut x = -r;
        let mut y = 0isize;
        let mut err = 2 - 2 * r; /* II. Quadrant */
        while {
            self.buffer[Framebuffer::xy_to_i(
                (xMid as isize - x) as usize,
                (yMid as isize + y) as usize,
            )] = colour; /*   I. Quadrant */
            self.buffer[Framebuffer::xy_to_i(
                (xMid as isize - y) as usize,
                (yMid as isize - x) as usize,
            )] = colour; /*  II. Quadrant */
            self.buffer[Framebuffer::xy_to_i(
                (xMid as isize + x) as usize,
                (yMid as isize - y) as usize,
            )] = colour; /* III. Quadrant */
            self.buffer[Framebuffer::xy_to_i(
                (xMid as isize + y) as usize,
                (yMid as isize + x) as usize,
            )] = colour; /*  IV. Quadrant */
            r = err;
            if r <= y {
                y += 1;
                err += y * 2 + 1; /* e_xy+e_y < 0 */
            }
            if r > x || err > y {
                x += 1;
                err += x * 2 + 1; /* e_xy+e_x > 0 or no 2nd y-step */
            }

            x < 0
        } {}
    }

    #[inline]
    //see https://stackoverflow.com/a/12016968/4496839
    pub fn blend(&mut self, i: usize, colour: u32) {
        let background = self.buffer[i];
        let alpha = alpha!(colour) + 1;
        let inv_alpha = 256 - alpha!(colour);
        self.buffer[i] = colour!(
            (alpha * red!(colour) + inv_alpha * red!(background)) >> 8,
            (alpha * green!(colour) + inv_alpha * green!(background)) >> 8,
            (alpha * blue!(colour) + inv_alpha * blue!(background)) >> 8,
            0xFF
        );
    }

    #[inline]
    pub fn blend_xy(&mut self, x: usize, y: usize, colour: u32) {
        self.blend(Framebuffer::xy_to_i(x, y), colour);
    }

    //see http://members.chello.at/easyfilter/bresenham.c
    pub fn draw_circle(&mut self, xMid: usize, yMid: usize, radius: usize, colour: u32) {
        if xMid < radius || yMid < radius {
            if cfg!(debug_assertions) {
                console!(log, "draw_circle xMid < radius || yMid < radius");
            }

            return;
        }
        let xm = xMid as isize;
        let ym = yMid as isize;

        /* II. quadrant from bottom left to top right */
        let mut x: isize = -(radius as isize);
        let mut y: isize = 0;

        let mut alpha;

        /* error of 1.step */
        let mut err: isize = 2 - 2 * (radius as isize);

        //equivalent to 2 * radius - 1
        let diameter = 1 - err;
        while {
            /* get blend value of pixel */
            alpha = 255 * isize::abs(err - 2 * (x + y) - 2) / diameter;

            {
                let new_colour = set_alpha!(colour, 255 - (alpha as u32));

                /*   I. Quadrant */
                self.blend_xy((xm - x) as usize, (ym + y) as usize, new_colour);
                /*  II. Quadrant */
                self.blend_xy((xm - y) as usize, (ym - x) as usize, new_colour);
                /* III. Quadrant */
                self.blend_xy((xm + x) as usize, (ym - y) as usize, new_colour);
                /*  IV. Quadrant */
                self.blend_xy((xm + y) as usize, (ym + x) as usize, new_colour);
            }

            /* remember values */
            let e2 = err;
            let x2 = x;

            /* x step */
            if err + y > 0 {
                alpha = 255 * (err - 2 * x - 1) / diameter;

                /* outward pixel */
                if alpha < 256 {
                    let new_colour = set_alpha!(colour, 255 - (alpha as u32));

                    self.blend_xy((xm - x) as usize, (ym + y + 1) as usize, new_colour);
                    self.blend_xy((xm - y - 1) as usize, (ym - x) as usize, new_colour);
                    self.blend_xy((xm + x) as usize, (ym - y - 1) as usize, new_colour);
                    self.blend_xy((xm + y + 1) as usize, (ym + x) as usize, new_colour);
                }
                x += 1;
                err += x * 2 + 1;
            }

            /* y step */
            if e2 + x2 <= 0 {
                alpha = 255 * (2 * y + 3 - e2) / diameter;

                /* inward pixel */
                if alpha < 256 {
                    let new_colour = set_alpha!(colour, 255 - (alpha as u32));
                    self.blend_xy((xm - x2 - 1) as usize, (ym + y) as usize, new_colour);
                    self.blend_xy((xm - y) as usize, (ym - x2 - 1) as usize, new_colour);
                    self.blend_xy((xm + x2 + 1) as usize, (ym - y) as usize, new_colour);
                    self.blend_xy((xm + y) as usize, (ym + x2 + 1) as usize, new_colour);
                }
                y += 1;
                err += y * 2 + 1;
            }

            x < 0
        } {}
    }

    pub fn draw_filled_circle(&mut self, xMid: usize, yMid: usize, radius: usize, colour: u32) {
        if xMid < radius || yMid < radius {
            if cfg!(debug_assertions) {
                console!(log, "draw_filled_circle xMid < radius || yMid < radius");
            }

            return;
        }
        let xm = xMid as isize;
        let ym = yMid as isize;

        /* II. quadrant from bottom left to top right */
        let mut x: isize = -(radius as isize);
        let mut y: isize = 0;

        let mut alpha;

        /* error of 1.step */
        let mut err: isize = 2 - 2 * (radius as isize);

        //equivalent to 2 * radius - 1
        let diameter = 1 - err;
        while {
            /* get blend value of pixel */
            alpha = 255 * isize::abs(err - 2 * (x + y) - 2) / diameter;

            {
                let new_colour = set_alpha!(colour, 255 - (alpha as u32));

                /*   I. Quadrant */
                self.blend_xy((xm - x) as usize, (ym + y) as usize, new_colour);
                /*  II. Quadrant */
                self.blend_xy((xm - y) as usize, (ym - x) as usize, new_colour);
                /* III. Quadrant */
                self.blend_xy((xm + x) as usize, (ym - y) as usize, new_colour);
                /*  IV. Quadrant */
                self.blend_xy((xm + y) as usize, (ym + x) as usize, new_colour);
            }

            /* remember values */
            let e2 = err;
            let x2 = x;

            /* x step */
            if err + y > 0 {
                alpha = 255 * (err - 2 * x - 1) / diameter;

                /* outward pixel */
                if alpha < 256 {
                    let new_colour = set_alpha!(colour, 255 - (alpha as u32));

                    self.blend_xy((xm - x) as usize, (ym + y + 1) as usize, new_colour);
                    self.blend_xy((xm - y - 1) as usize, (ym - x) as usize, new_colour);
                    self.blend_xy((xm + x) as usize, (ym - y - 1) as usize, new_colour);
                    self.blend_xy((xm + y + 1) as usize, (ym + x) as usize, new_colour);
                }
                x += 1;
                err += x * 2 + 1;
            }

            /* y step */
            if e2 + x2 <= 0 {
                /* inward pixels */

                let mut current_x;
                let mut current_y;

                current_x = (xm - x2 - 1) as usize;
                current_y = (ym + y) as usize;
                while current_x > xMid || current_y > yMid {
                    self.buffer[Framebuffer::xy_to_i(current_x, current_y)] = colour;

                    current_x -= 1;
                    current_y -= 1;
                }

                current_x = (xm + y) as usize;
                current_y = (ym + x2 + 1) as usize;
                while current_x > xMid || current_y < yMid {
                    self.buffer[Framebuffer::xy_to_i(current_x, current_y)] = colour;

                    current_x -= 1;
                    current_y += 1;
                }

                current_x = (xm - y) as usize;
                current_y = (ym - x2 - 1) as usize;
                while current_x < xMid || current_y > yMid {
                    self.buffer[Framebuffer::xy_to_i(current_x, current_y)] = colour;

                    current_x += 1;
                    current_y -= 1;
                }

                current_x = (xm + x2 + 1) as usize;
                current_y = (ym - y) as usize;
                while current_x < xMid || current_y < yMid {
                    self.buffer[Framebuffer::xy_to_i(current_x, current_y)] = colour;

                    current_x += 1;
                    current_y += 1;
                }

                y += 1;
                err += y * 2 + 1;
            }

            x < 0
        } {}

        self.buffer[Framebuffer::xy_to_i(xMid, yMid)] = colour;
    }

    pub fn sspr(
        &mut self,
        sprite_x: u8,
        sprite_y: u8,
        sprite_w: u8,
        sprite_h: u8,
        display_x: u8,
        display_y: u8,
    ) {
        const S_WIDTH: usize = GFX_WIDTH as usize;
        const D_WIDTH: usize = SCREEN_WIDTH as usize;

        let s_w = sprite_w as usize;
        let s_h = sprite_h as usize;

        let s_x = sprite_x as usize;
        let s_y = sprite_y as usize;

        let d_x = display_x as usize;
        let d_y = display_y as usize;

        let d_x_max = d_x + s_w;
        let d_y_max = d_y + s_h;

        let mut current_s_y = s_y;
        for y in d_y..d_y_max {
            let mut current_s_x = s_x;
            for x in d_x..d_x_max {
                let colour = GFX[current_s_x + current_s_y * S_WIDTH] as usize;
                //make purple transparent
                if colour != 2 {
                    let index = x + y * D_WIDTH;
                    if index < self.buffer.len() {
                        self.buffer[index] = PALETTE[colour];
                    }
                }
                current_s_x += 1;
            }
            current_s_y += 1;
        }
    }

    pub fn sspr_flip_both(
        &mut self,
        sprite_x: u8,
        sprite_y: u8,
        sprite_w: u8,
        sprite_h: u8,
        display_x: u8,
        display_y: u8,
    ) {
        const S_WIDTH: usize = GFX_WIDTH as usize;
        const D_WIDTH: usize = SCREEN_WIDTH as usize;

        let s_w = sprite_w as usize;
        let s_h = sprite_h as usize;

        let s_x = sprite_x as usize;
        let s_y = sprite_y as usize;

        let d_x = display_x as usize;
        let d_y = display_y as usize;

        let d_x_max = d_x + s_w;
        let d_y_max = d_y + s_h;

        let mut current_s_y = s_y + s_h - 1;
        for y in d_y..d_y_max {
            let mut current_s_x = s_x + s_w - 1;
            for x in d_x..d_x_max {
                let colour = GFX[current_s_x + current_s_y * S_WIDTH] as usize;
                //make purple transparent
                if colour != 2 {
                    let index = x + y * D_WIDTH;
                    if index < self.buffer.len() {
                        self.buffer[index] = PALETTE[colour];
                    }
                }
                current_s_x -= 1;
            }
            current_s_y -= 1;
        }
    }

    pub fn spr(&mut self, sprite_number: u8, x: u8, y: u8) {
        let (sprite_x, sprite_y) = get_sprite_xy(sprite_number);
        self.sspr(sprite_x, sprite_y, 8, 8, x, y);
    }

    pub fn spr_flip_both(&mut self, sprite_number: u8, x: u8, y: u8) {
        let (sprite_x, sprite_y) = get_sprite_xy(sprite_number);
        self.sspr_flip_both(sprite_x, sprite_y, 8, 8, x, y);
    }

    pub fn draw_map(&mut self) {
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                self.spr(MAP[x + y * MAP_WIDTH], x as u8 * 8, y as u8 * 8);
            }
        }
    }

    pub fn print(&mut self, string: &str, mut x: u8, y: u8, colour: u8) {
        for c in string.bytes() {
            let (sprite_x, sprite_y) = get_char_xy(c);
            self.print_char(sprite_x, sprite_y, 8, 8, x, y, colour);
            x = x.saturating_add(4);
        }
    }

    fn print_char(
        &mut self,
        sprite_x: u8,
        sprite_y: u8,
        sprite_w: u8,
        sprite_h: u8,
        display_x: u8,
        display_y: u8,
        colour: u8,
    ) {
        const S_WIDTH: usize = FONT_WIDTH as usize;
        const D_WIDTH: usize = SCREEN_WIDTH as usize;

        let s_w = sprite_w as usize;
        let s_h = sprite_h as usize;

        let s_x = sprite_x as usize;
        let s_y = sprite_y as usize;

        let d_x = display_x as usize;
        let d_y = display_y as usize;

        let d_x_max = d_x + s_w;
        let d_y_max = d_y + s_h;

        let mut current_s_y = s_y;
        for y in d_y..d_y_max {
            let mut current_s_x = s_x;
            for x in d_x..d_x_max {
                let foxt_pixel_colour = FONT[current_s_x + current_s_y * S_WIDTH] as usize;
                //make black transparent
                if foxt_pixel_colour != 0 {
                    let index = x + y * D_WIDTH;
                    if index < self.buffer.len() {
                        self.buffer[index] = PALETTE[colour as usize & 15];
                    }
                }
                current_s_x += 1;
            }
            current_s_y += 1;
        }
    }
}

pub fn get_sprite_xy(sprite_number: u8) -> (u8, u8) {
    const SPRITES_PER_ROW: u8 = GFX_WIDTH as u8 / 8;

    (
        (sprite_number % SPRITES_PER_ROW) * 8,
        (sprite_number / SPRITES_PER_ROW) * 8,
    )
}

pub fn get_char_xy(sprite_number: u8) -> (u8, u8) {
    const SPRITES_PER_ROW: u8 = FONT_WIDTH as u8 / 8;

    (
        (sprite_number % SPRITES_PER_ROW) * 8,
        (sprite_number / SPRITES_PER_ROW) * 8,
    )
}

impl Default for Framebuffer {
    fn default() -> Self {
        let mut buffer = Vec::new();
        buffer.resize(SCREEN_WIDTH * SCREEN_HEIGHT, PALETTE[0]);

        Framebuffer { buffer }
    }
}

use std::cmp::min;

pub fn draw_winning_screen(framebuffer: &mut Framebuffer) {
    let mut colour_index = 8;
    let mut w = SCREEN_WIDTH;
    let mut h = SCREEN_HEIGHT;

    let smaller_side = min(SCREEN_WIDTH, SCREEN_HEIGHT);
    let layers = (smaller_side - smaller_side / 3) / 2;

    for corner in 0..layers {
        let colour = PALETTE[colour_index];
        framebuffer.draw_rect(corner, corner, w, h, colour);

        if w > 2 {
            w -= 2;
        }
        if h > 2 {
            h -= 2;
        }

        colour_index = (colour_index + 1) & 0xF;
    }
}
