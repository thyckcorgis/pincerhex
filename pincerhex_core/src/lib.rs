#![no_std]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(
    clippy::implicit_return,
    clippy::question_mark_used,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]

#[macro_use]
extern crate alloc;

mod board;
mod eval;
mod tile;

pub use board::{Board, Error as BoardError};
pub use eval::{PotentialEvaluator, STARTING_COLOUR};
pub use tile::{Colour, Error as TileError, Move, PieceState, Tile};

pub trait Rand {
    fn in_range(&mut self, a: i8, b: i8) -> i8;
    fn next(&mut self) -> f32;
}

const NO_SWAP_CHANCE: i8 = 2;

pub fn should_swap(r: i8, c: i8, size: i8, rand: &mut impl Rand) -> bool {
    ((r + c < 2) || (r + c > 2 * size - 4))
        || (((r + c == 2) || (r + c == 2 * size - 4)) && rand.in_range(0, NO_SWAP_CHANCE) == 0)
}

pub fn first_move(size: i8, rand: &mut impl Rand) -> (i8, i8) {
    let (mut i, mut j) = (
        rand.in_range(0, size / 2 - 1),
        rand.in_range(0, size / 2 - 1),
    );
    if rand.in_range(0, 2) == 0 {
        i = size - 1 - i;
        j = size - 1 - j;
    }
    (i, j)
}
