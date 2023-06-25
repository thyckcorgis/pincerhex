#![no_std]
extern crate alloc;
use alloc::string::String;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use pincerhex_core::{
    first_move, should_swap, Board, Colour, PieceState, PotentialEvaluator, Tile,
};
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

#[inline]
fn get_bot_colour(bot_is_white: bool) -> Colour {
    if bot_is_white {
        Colour::White
    } else {
        Colour::Black
    }
}

/// Returns the string representation of a blank board.
///
/// # Arguments
///
/// * `size` - Size of the board from 1 to 26. Recommended size is 10.
#[wasm_bindgen]
pub fn get_board(size: i8) -> String {
    Board::new(size).get_compressed()
}

/// Play the bot's first move. Returns the string representation of the board.
///
/// # Arguments
///
/// * `bot_is_white` - Whether the bot starts as white.
/// * `size` - Size of the board from 1 to 26. Recommended size is 10.
/// * `seed` - 64-bit seed used for random number generation.
#[wasm_bindgen]
pub fn get_first_move(bot_is_white: bool, size: i8, seed: u64) -> String {
    let mut rng = WasmRng(SmallRng::seed_from_u64(seed));
    let mut board = Board::new(size);
    let (r, c) = first_move(board.size, &mut rng);
    let colour = get_bot_colour(bot_is_white);
    board
        .set_tile(Tile::Regular(r, c), PieceState::Colour(colour))
        .expect("to be valid");
    board.get_compressed()
}

/// Play a move. Since we want this library to be stateless all state has to be passed to this
/// function.
///
/// # Arguments
///
/// * `bot_is_white` - Whether the bot's current colour is white.
/// * `bot_started_white` - Whether the bot's started as white.
/// * `player_move` - a 16-bit number containing the column index in the LSB and the row index in
/// the MSB.
/// * `board` - String representation of the board. Can be received from `get_first_move` if
/// the bot is starting or `get_board` if the player is starting.
/// * `move_count` - Number of moves played in the game so far.
/// * `seed` - 64-bit seed used for random number generation.
#[wasm_bindgen]
pub fn pincerhex_move(
    bot_is_white: bool,
    bot_started_white: bool,
    player_move: u16,
    board: &str,
    move_count: u16,
    seed: u64,
) -> u32 {
    let mut rng = WasmRng(SmallRng::seed_from_u64(seed));
    let colour = get_bot_colour(bot_is_white);
    let starting = get_bot_colour(bot_started_white);
    let (r, c) = ((player_move >> 8) as i8, (player_move & 0xff) as i8);
    let board = Board::from(board);

    match move_count {
        1 if should_swap(r, c, board.size, &mut rng) => Move::Swap,
        count => Move::Regular(
            PotentialEvaluator::new(&board, colour, starting)
                .evaluate()
                .get_best_move(count, &mut rng),
        ),
    }
    .into()
}
