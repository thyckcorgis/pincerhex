#![no_std]

use rand::{rngs::SmallRng, Rng, SeedableRng};

use pincerhex_core::{first_move, should_swap, Board, Colour, PotentialEvaluator};
use wasm_bindgen::prelude::wasm_bindgen;

enum Move {
    Regular((i8, i8)),
    Swap,
}

impl From<Move> for u32 {
    fn from(value: Move) -> Self {
        match value {
            Move::Regular((r, c)) => ((r as u32) << 8) | (c as u32),
            Move::Swap => 0xffff0000,
        }
    }
}

struct WasmRng(SmallRng);

impl pincerhex_core::Rand for WasmRng {
    fn in_range(&mut self, a: i8, b: i8) -> i8 {
        self.0.gen_range(a..b)
    }

    fn next(&mut self) -> f32 {
        self.0.gen::<f32>()
    }
}

#[wasm_bindgen]
pub fn pincerhex_move(
    bot_is_white: bool,
    player_move: u16,
    board: &str,
    move_count: u16,
    seed: u64,
) -> u32 {
    let mut rng = WasmRng(SmallRng::seed_from_u64(seed));
    let colour = if bot_is_white {
        Colour::White
    } else {
        Colour::Black
    };
    let (r, c) = ((player_move >> 8) as i8, (player_move & 0xff) as i8);
    let board = Board::from(board);

    match move_count {
        0 => Move::Regular(first_move(board.size, &mut rng)),
        1 if should_swap(r, c, board.size, &mut rng) => Move::Swap,
        count => Move::Regular(
            PotentialEvaluator::new(&board, colour)
                .evaluate()
                .get_best_move(count, &mut rng),
        ),
    }
    .into()
}
