mod ai;
mod state;
mod union_find;

pub use ai::{BotError, HexBot};
pub use pot_eval::STARTING_COLOUR;
pub use state::Error as StateError;
pub use tile_state::{
    board::Error as BoardError,
    tile::{Colour, Error as TileError, Move, PieceState},
};

pub enum Winner {
    Bot,
    Opponent,
}
