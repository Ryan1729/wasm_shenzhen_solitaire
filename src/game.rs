use common::*;

#[inline]
pub fn update_and_render(framebuffer: &mut Framebuffer, state: &mut GameState, input: Input) {
    framebuffer.clear();

    draw_winning_screen(framebuffer);

    if input.pressed_this_frame(Button::A) {
        console!(log, &framebuffer.buffer);
    }
}
