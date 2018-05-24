use common::*;

fn update(state: &mut GameState, input: Input) {}

fn draw(framebuffer: &mut Framebuffer, state: &GameState) {
    framebuffer.clear();

    framebuffer.sspr_flip_both(0, 0, 97, GFX_HEIGHT as _, 0, 0);

    framebuffer.sspr(0, 8, 16, 24, 0, GFX_HEIGHT as u8);

    framebuffer.spr(7, 0, GFX_HEIGHT as u8 + 1);
    framebuffer.spr_flip_both(7, 9, GFX_HEIGHT as u8 + 1 + 16);
}

#[inline]
pub fn update_and_render(framebuffer: &mut Framebuffer, state: &mut GameState, input: Input) {
    update(state, input);

    draw(framebuffer, &state);

    if input.pressed_this_frame(Button::A) {
        console!(log, &framebuffer.buffer);
    }
}
