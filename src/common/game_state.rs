extern crate rand;

use self::rand::{Rng, SeedableRng};
use std::mem;
use stdweb::web::Date;

use inner_common::*;

impl GameState {
    pub fn new() -> GameState {
        let mut cells: [Vec<u8>; 16] = Default::default();

        let mut deck = Vec::with_capacity(3 * (START_OF_TABLEAU as usize + 4) + 1);

        for i in 1..=MAX_SUIT_NUM {
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
            // let time = Date::new().get_time();
            //
            // mem::transmute::<[f64; 2], [u32; 4]>([time, 1.0 / time])

            //known non-autoplay seed
            //[593227776, 1115044417, 342636230, 1030162643]

            //known autoplay seed
            [614731776, 1115044419, 8528335, 1030162641]
        };

        console!(log, format!("{:?}", seed));
        let mut rng = rand::XorShiftRng::from_seed(seed);

        let mut deckpos = START_OF_TABLEAU;
        while deck.len() > 0 {
            let index = rng.gen_range(0, deck.len());
            cells[deckpos as usize].push(deck.swap_remove(index));

            deckpos = if deckpos >= CELLS_MAX_INDEX {
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
