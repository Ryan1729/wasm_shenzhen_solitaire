extern crate rand;

use self::rand::{Rng, SeedableRng};
use std::mem;
use stdweb::web::Date;

use inner_common::*;

impl GameState {
    pub fn new() -> GameState {
        let mut cells: [Vec<u8>; 16] = Default::default();

        let mut deck = Vec::with_capacity(3 * (START_OF_TABLEAU as usize + 4) + 1);
        let mut deckpos = START_OF_TABLEAU;

        for i in 1..=START_OF_TABLEAU {
            deck.push(i);
            deck.push(i + 10);
            deck.push(i + 20);
        }

        for _ in 1..=4 {
            deck.push(0);
            deck.push(10);
            deck.push(20);
        }

        deck.push(30);

        let seed = unsafe {
            let time = Date::new().get_time();

            mem::transmute::<[f64; 2], [u32; 4]>([time, 1.0 / time])
        };
        let mut rng = rand::XorShiftRng::from_seed(seed);

        while deck.len() > 0 {
            let index = rng.gen_range(0, deck.len());
            cells[deckpos as usize].push(deck.swap_remove(index));

            deckpos = if deckpos >= CELLS_LEN - 1 {
                START_OF_TABLEAU
            } else {
                deckpos + 1
            };
        }

        GameState {
            cells,
            wins: 0,
            win_done: false,
            selectdrop: false,
            selectpos: START_OF_TABLEAU,
            selectdepth: 0,
            grabpos: 1,
            grabdepth: 0,
            movetimer: 0,
        }
    }
}
