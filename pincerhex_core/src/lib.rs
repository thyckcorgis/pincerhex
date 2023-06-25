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
mod board;
mod eval;
mod state;
mod tile;
mod union_find;

pub use ai::{BotError, HexBot};
pub use board::Error as BoardError;
pub use eval::STARTING_COLOUR;
pub use state::Error as StateError;
pub use tile::{Colour, Error as TileError, Move, PieceState};

pub enum Winner {
    Bot,
    Opponent,
}
