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

extern crate alloc;

mod ai;

pub use ai::{BotError, HexBot};
pub use pincerhex_core::{Colour, Move, PieceState};
pub use pincerhex_state::Winner;
use rand::{rngs::ThreadRng, Rng};

struct StdRng(ThreadRng);

impl pincerhex_core::Rand for StdRng {
    fn in_range(&mut self, a: i8, b: i8) -> i8 {
        self.0.gen_range(a..b)
    }

    fn next(&mut self) -> f32 {
        self.0.gen::<f32>()
    }
}
