#![no_std]

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
