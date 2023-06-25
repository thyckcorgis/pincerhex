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

mod ai;
mod state;
mod union_find;

pub use ai::{BotError, HexBot};
pub use pincerhex_core::{Colour, Move, PieceState};
use rand::{rngs::ThreadRng, Rng};

pub enum Winner {
    Bot,
    Opponent,
}

struct StdRng(ThreadRng);

impl pincerhex_core::Rand for StdRng {
    fn in_range(&mut self, a: i8, b: i8) -> i8 {
        self.0.gen_range(a..b)
    }

    fn next(&mut self) -> f32 {
        self.0.gen::<f32>()
    }
}
