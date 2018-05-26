use common::*;

use std::cmp::{max, min};

macro_rules! last_unchecked {
    ($vec: expr) => {
        $vec[$vec.len() - 1]
    }
}

fn update(state: &mut GameState, input: Input) {
    if state.movetimer > 0 {
        state.movetimer -= 1;
    }

	if state.movetimer == 0 {
		if automove(state) {
			state.movetimer = MOVE_TIMER_MAX;
		} else {
			if input.pressed_this_frame(Button::Left) {
				let oldinv = max(1, state.cells[state.selectpos as usize].len() as u8 - state.selectdepth);
				state.selectpos = if state.selectpos == 0 {
					 START_OF_TABLEAU - 1
                }else if state.selectpos == START_OF_TABLEAU {
					CELLS_MAX_INDEX
				}else{
					state.selectpos - 1
				};
				state.selectdepth = if state.selectdrop {
					 0
				} else {
                    let len = state.cells[state.selectpos as usize].len() as u8;
					 min( max(0, len - oldinv), len)
				};
			} else if input.pressed_this_frame(Button::Right) {
                let oldinv = max(1, state.cells[state.selectpos as usize].len() as u8 - state.selectdepth);
                state.selectpos = if state.selectpos == START_OF_TABLEAU - 1 {
					 0
                }else if state.selectpos >= CELLS_MAX_INDEX {
					START_OF_TABLEAU
				}else{
					state.selectpos + 1
				};
				state.selectdepth = if state.selectdrop {
					 0
				} else {
                    let len = state.cells[state.selectpos as usize].len() as u8;
					 min( max(0, len - oldinv), len)
				};
            } else if input.pressed_this_frame(Button::Up) {
				let changepos = if state.selectpos == BUTTON_COLUMN {
					 state.selectdepth >= 2
				} else {
                    let len = state.cells[state.selectpos as usize].len();
                    len == 0
                    || state.selectdepth >= len as u8 - 1
                    || state.selectdrop
				};

				if changepos {
					state.selectpos = if state.selectpos > START_OF_TABLEAU {
						state.selectpos - START_OF_TABLEAU
					} else {
						state.selectpos + START_OF_TABLEAU
					};
					state.selectdepth = 0;
				} else {
					state.selectdepth += 1;
				}
			} else if input.pressed_this_frame(Button::Down) {
				if state.selectdepth == 0 {
					state.selectpos = if state.selectpos > START_OF_TABLEAU {
						state.selectpos - START_OF_TABLEAU
					} else {
						state.selectpos + START_OF_TABLEAU
					};
                    let len = state.cells[state.selectpos as usize].len();
					state.selectdepth = if len > 0 && !state.selectdrop {
						 len as u8 - 1
					} else if state.selectpos == BUTTON_COLUMN {
						2
					} else {
						0
					};
				} else {
					state.selectdepth = state.selectdepth - 1;
				}
			} else if input.pressed_this_frame(Button::A) {
				if state.selectpos == BUTTON_COLUMN {
					if canmovedragons(state, state.selectdepth) {
						movedragons(state, state.selectdepth);
						state.selectdrop = false;
						state.movetimer = MOVE_TIMER_MAX;
					}
				} else {
					if state.selectdrop {
						if candrop(state.grabpos, state.grabdepth, state.selectpos) {
							movecards(state, state.grabpos, state.grabdepth, state.selectpos);
							state.selectdrop = false;
							state.movetimer = MOVE_TIMER_MAX;
						}
					} else if cangrab(state.selectpos, state.selectdepth) {
						state.grabpos = state.selectpos;
						state.grabdepth = state.selectdepth;
						state.selectdrop = true;
					}
				}
			} else if input.pressed_this_frame(Button::B) {
				state.selectdrop = false;
			}
		}
	}

	if haswon(state) && !state.win_done {
		state.wins += 1;
		state.win_done = true;
	}
}

fn getsuit(card: u8) -> u8 {
	if card >= FLOWER_CARD {
		3
	} else if card >= FIRST_BLACK_CARD {
		2
	} else if card >= FIRST_GREEN_CARD {
		1
	} else {
		0
	}
}

fn getcardnum(card: u8) -> u8 {
	card - (getsuit(card) * 10)
}

fn movecards(state: &mut GameState, grabpos: u8, grabdepth: u8, droppos: u8) {
    let grabpos = grabpos as usize;
    let grabdepth = grabdepth as usize;
    let droppos = droppos as usize;
	if droppos <= 8 {
		state.cells[droppos].insert(0, last_unchecked!(state.cells[grabpos]));
		for i in state.cells[grabpos].len()..=state.cells[grabpos].len()-grabdepth {
			state.cells[grabpos].remove(i);
		}
	} else {
		for i in state.cells[grabpos].len()-grabdepth..=state.cells[grabpos].len() {
            let i = i as usize;
			state.cells[droppos].push(state.cells[grabpos].remove(i));
		}
	}
}

fn canmovedragons(state: &GameState, suit: u8) -> bool {
	let mut count = 0;
	for i in 0..=CELLS_MAX_INDEX {
        let i = i as usize;
		if state.cells[i].len() > 0 && last_unchecked!(state.cells[i]) == suit * 10 {
			count += 1;
		}
	}

	if count < 4 {
        return false;
    }

	for i in 0..BUTTON_COLUMN {
        let i = i as usize;
		if state.cells[i].len() == 0
        || last_unchecked!(state.cells[i]) == suit * 10 {
			return true;
		}
	}
	return false;
}

fn movedragons(state: &mut GameState, suit: u8) {
	let mut moveto = None;

	for i in 0..BUTTON_COLUMN {
        let i = i as usize;
		if state.cells[i].len() == 0
        && last_unchecked!(state.cells[i]) == suit * 10
		&& moveto.is_none() {
			moveto = Some(i);
		}
	}
	if moveto.is_none() {
		for i in 0..BUTTON_COLUMN {
            let i = i as usize;
			if state.cells[i].len() == 0 {
				moveto = Some(i);
                break;
			}
		}
	}

	for i in 0..=CELLS_MAX_INDEX {
        let i = i as usize;
		if state.cells[i].len() != 0
			&& last_unchecked!(state.cells[i]) == suit * 10 {
			state.cells[i].pop();
		}
	}

    if let Some(moveto) = moveto {
        let moveto = moveto as usize;
        state.cells[moveto].push(CARD_BACK);
    }
}

fn haswon(state: &GameState) -> bool {
	for i in START_OF_TABLEAU..=CELLS_MAX_INDEX {
        let i = i as usize;
		if state.cells[i].len() > 0 {
			return false;
		}
	}
	return true;
}

fn automove(state: &mut GameState) -> bool {
	let mut minfval = None;

	for i in START_OF_FOUNDATIONS..START_OF_TABLEAU {
        let i = i as usize;
		let val = if state.cells[i].len() > 0 {
			let card = last_unchecked!(state.cells[i]);
			getcardnum(card)
		} else {
            0
        };
		if minfval.map(|v| val < v).unwrap_or(true) {
			minfval = Some(val);
		}
	}

	for i in 0..=CELLS_MAX_INDEX {
		if (i < BUTTON_COLUMN || i >= START_OF_TABLEAU)
        && state.cells[i as usize].len() > 0 {
			let card = last_unchecked!(state.cells[i as usize]);
			if card == FLOWER_CARD {
				movecards(state, i, 0, FLOWER_FOUNDATION);
				return true;
			} else if getcardnum(card) == minfval.unwrap_or(255).wrapping_add(1) && card != CARD_BACK {
				for i2 in START_OF_FOUNDATIONS..START_OF_TABLEAU {
					if state.cells[i2 as usize].len() > 0 {
						let card2 = state.cells[i2 as usize].len() as u8;
						if getsuit(card2) == getsuit(card) {
							movecards(state, i, 0, i2);
							return true;
						}
					}
				}
				for i2 in START_OF_FOUNDATIONS..START_OF_TABLEAU {
	                if state.cells[i2 as usize].len() == 0 {
                        movecards(state, i, 0, i2);
                        return true;
                    }
                }
            }
        }
    }

	return false;
}

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
        for cell in state.cells.iter() {
            console!(log, cell);
        }
    }
}
