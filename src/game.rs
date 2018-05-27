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
						movedragons(state);
						state.selectdrop = false;
						state.movetimer = MOVE_TIMER_MAX;
					}
				} else {
					if state.selectdrop {
						if candrop(&state.cells, state.grabpos, state.grabdepth, state.selectpos) {
                            let GameState{grabpos, grabdepth, selectpos, ..} = state;
							movecards(state, *grabpos, *grabdepth, *selectpos);
							state.selectdrop = false;
							state.movetimer = MOVE_TIMER_MAX;
						}
					} else if cangrab(&state.cells, state.selectpos, state.selectdepth) {
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

fn getselection(cells: &Cells, pos: u8, depth: u8) -> Vec<u8> {
    let pos = pos as usize;
    let depth = depth as usize;

	let mut output = Vec::with_capacity(depth);
	for i in 0..depth {
		let index = cells[pos].len() - (depth + 1) + i;
		output.push(cells[pos][index]);
	}
	return output;
}

fn cangrab(cells: &Cells, pos: u8, depth: u8) -> bool {
	let selection = getselection(cells, pos, depth);

	if selection.len() == 0
    || (pos >= FLOWER_FOUNDATION && pos < START_OF_TABLEAU) {
		return false;
	}

    let mut lastsuit = 255;
    let mut lastnum = 255;
    let mut first = true;

	for &card in selection.iter() {
        if card == CARD_BACK {
			return false;
		}

		let suit = getsuit(card);
		let num = getcardnum(card);

        if !first {
			if suit == lastsuit
            || num == 0
            || num != lastnum - 1 {
				return false;
			}
		}
		lastsuit = suit;
		lastnum = num;
		first = false;
	}

	return true;
}

fn candrop(cells: &Cells, grabpos: u8, grabdepth: u8, droppos: u8) -> bool {
    let grabpos = grabpos as usize;
    let grabdepth = grabdepth as usize;

	let grabcard = {
        let len = cells[grabpos].len();
        if len < grabdepth {
            return false;
        }

        cells[grabpos][len - grabdepth]
    };

	if droppos < BUTTON_COLUMN {
		return cells[droppos as usize].len() == 0 && grabdepth == 0;
	} else if droppos >= BUTTON_COLUMN && droppos <= FLOWER_FOUNDATION {
		return false;
	} else if droppos >= START_OF_FOUNDATIONS && droppos < START_OF_TABLEAU {
        let droppos = droppos as usize;
		if grabdepth == 0 {
			if cells[droppos].len() == 0 {
				if getcardnum(grabcard) == 1 {
					return true;
				}
			} else {
				let dropcard = last_unchecked!(cells[droppos]);
				if getsuit(grabcard) == getsuit(dropcard)
				&& getcardnum(grabcard) != 0
				&& getcardnum(grabcard) == getcardnum(dropcard) + 1 {
					return true;
				}
			}
		}
		return false;
	} else {
        let droppos = droppos as usize;
		if cells[droppos].len() == 0 {
			return true;
		} else {
			let dropcard = last_unchecked!(cells[droppos]);
			if getsuit(grabcard) != getsuit(dropcard)
			&& getcardnum(grabcard) != 0
			&& getcardnum(grabcard) == getcardnum(dropcard) - 1 {
				return true;
			}
		}
		return false;
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
        let last = last_unchecked!(state.cells[grabpos]);
		state.cells[droppos].insert(0, last);
		for i in state.cells[grabpos].len()..=state.cells[grabpos].len()-grabdepth {
			state.cells[grabpos].remove(i);
		}
	} else {
		for i in state.cells[grabpos].len()-grabdepth..=state.cells[grabpos].len() {
            let i = i as usize;
            let removed = state.cells[grabpos].remove(i);
			state.cells[droppos].push(removed);
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

fn movedragons(state: &mut GameState) {
    let suit = state.selectpos;
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
    let min_free_card_num = {
        let mut min_foundation_card_num = None;

    	for i in START_OF_FOUNDATIONS..START_OF_TABLEAU {
            let i = i as usize;
    		let val = if state.cells[i].len() > 0 {
    			let card = last_unchecked!(state.cells[i]);
    			getcardnum(card)
    		} else {
                0
            };
    		if min_foundation_card_num.map(|v| val < v).unwrap_or(true) {
    			min_foundation_card_num = Some(val);
    		}
    	}

        min_foundation_card_num.unwrap_or(255).wrapping_add(1)
    };

	for i in 0..=CELLS_MAX_INDEX {
		if (i < BUTTON_COLUMN || i >= START_OF_TABLEAU)
        && state.cells[i as usize].len() > 0 {
			let card = last_unchecked!(state.cells[i as usize]);
			if card == FLOWER_CARD {
				movecards(state, i, 0, FLOWER_FOUNDATION);
				return true;
			} else if getcardnum(card) == min_free_card_num && card != CARD_BACK {
                let suit = getsuit(card);
				for i2 in START_OF_FOUNDATIONS..START_OF_TABLEAU {
					if state.cells[i2 as usize].len() > 0 {
						let card2 = state.cells[i2 as usize].len() as u8;
						if getsuit(card2) == suit {
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
    framebuffer.draw_map();

    framebuffer.sspr(0, 8, 16, 24, 0, GFX_HEIGHT as u8);

    framebuffer.spr(7, 0, GFX_HEIGHT as u8 + 1);
    framebuffer.spr_flip_both(7, 9, GFX_HEIGHT as u8 + 1 + 16);
}

#[inline]
pub fn update_and_render(framebuffer: &mut Framebuffer, state: &mut GameState, input: Input) {
    //update(state, input);

    draw(framebuffer, &state);

    if input.pressed_this_frame(Button::A) {
        for cell in state.cells.iter() {
            console!(log, cell);
        }
    }
}
