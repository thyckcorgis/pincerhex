use alloc::vec::Vec;

use pincerhex_state::State;

use crate::board::Piece;

pub struct PincerhexState(pub State);

#[derive(serde::Serialize, serde::Deserialize)]
struct SerializedState {
    active: Piece,
    pieces: Vec<(i8, i8, Piece)>,
    size: i8,
}

impl From<SerializedState> for PincerhexState {
    fn from(value: SerializedState) -> Self {
        use pincerhex_core::{PieceState, Tile};
        let mut state = State::new(value.size);
        state.set_to_play(value.active.into());
        for &(r, c, colour) in value.pieces.iter() {
            state
                .place_piece(Tile::Regular(r, c), PieceState::Colour(colour.into()))
                .unwrap();
        }
        Self(state)
    }
}

impl serde::Serialize for PincerhexState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use pincerhex_core::{PieceState, Tile};
        let board = self.0.get_board();
        let active: Piece = self.0.active().into();
        let mut pieces = Vec::new();
        for p in board.iter() {
            if let (Tile::Regular(r, c), PieceState::Colour(colour)) = p {
                pieces.push((r, c, colour.into()));
            }
        }

        SerializedState {
            active,
            pieces,
            size: board.size,
        }
        .serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for PincerhexState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        SerializedState::deserialize(deserializer).map(|x| x.into())
    }
}
