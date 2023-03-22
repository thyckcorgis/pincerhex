mod ai;
mod board;
mod potential;
mod state;
mod tile;
mod union_find;

pub use ai::{BotError, HexBot};
pub use board::Error as BoardError;
pub use state::Error as StateError;
pub use tile::{Colour, Error as TileError, Move, PieceState};

pub static mut STARTING_COLOUR: Colour = Colour::Black;

pub enum Winner {
    Bot,
    Opponent,
}
