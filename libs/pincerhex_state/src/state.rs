use alloc::string::String;
use pincerhex_core::{should_swap, Board, BoardError, Colour, PieceState, Rand, Tile};

use crate::{union_find::UnionFind, Winner};

pub struct State {
    size: i8,
    board: Board,
    to_play: Colour,
    groups: Groups,
}

#[derive(Debug)]
pub enum Error {
    TileNotEmpty,
    InvalidTile,
    Board(BoardError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TileNotEmpty => write!(f, "tile not empty"),
            Self::InvalidTile => write!(f, "invalid tile"),
            Self::Board(b) => match b {
                BoardError::NotInRange => write!(f, "not in range"),
            },
        }
    }
}

pub const DEFAULT_SIZE: i8 = 10;

impl From<BoardError> for Error {
    fn from(value: BoardError) -> Self {
        Self::Board(value)
    }
}

impl State {
    pub fn new(size: i8) -> Self {
        Self {
            size,
            board: Board::new(size),
            ..Default::default()
        }
    }

    pub const fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn should_swap(&self, rng: &mut impl Rand) -> bool {
        for i in self.board.iter() {
            if let (Tile::Regular(r, c), PieceState::Colour(_)) = i {
                return should_swap(r, c, self.size, rng);
            }
        }
        false
    }

    #[allow(dead_code)]
    pub fn swap_pieces(&mut self) -> Result<(), Error> {
        self.board.swap_pieces()?;
        Ok(())
    }

    pub fn get_compressed(&self) -> String {
        self.board.get_compressed()
    }

    pub fn get_pretty(&self) -> String {
        format!("{}", self.board)
    }

    fn is_connected(&mut self, colour: Colour) -> bool {
        self.groups
            .get_mut(colour)
            .connected(Tile::Edge1, Tile::Edge2)
    }

    pub fn get_winner(&mut self, colour: Colour) -> Option<Winner> {
        self.check_win().map(|w| {
            if w == colour {
                Some(Winner::Bot)
            } else {
                Some(Winner::Opponent)
            }
        })?
    }

    pub fn check_win(&mut self) -> Option<Colour> {
        if self.is_connected(Colour::White) {
            Some(Colour::White)
        } else if self.is_connected(Colour::Black) {
            Some(Colour::Black)
        } else {
            None
        }
    }

    fn set_piece(&mut self, t: Tile, s: PieceState) -> Result<(), Error> {
        self.board.set_tile(t, s)?;
        if let PieceState::Colour(c) = s {
            self.groups.join(t, c, &self.board);
            self.to_play = c.opponent();
        }
        Ok(())
    }

    fn replace_piece(&mut self, t: Tile, old: Colour, new: PieceState) -> Result<(), Error> {
        self.board.set_tile(t, new)?;
        self.groups.0[old.group_idx()] = UnionFind::new(self.size.try_into().unwrap());
        for i in self.board.iter() {
            if matches!(i.1, PieceState::Colour(c) if c == old) {
                self.groups.join(i.0, old, &self.board);
            }
        }
        Ok(())
    }

    pub fn place_piece(&mut self, t: Tile, s: PieceState) -> Result<(), Error> {
        if let PieceState::Colour(c) = self.board.get_tile(t).ok_or(Error::InvalidTile)? {
            match (s, c) {
                (PieceState::Empty, c) => self.replace_piece(t, c, s),
                (PieceState::Colour(to_place), c) if to_place != c => self.replace_piece(t, c, s),
                (_, _) => Ok(()),
            }?;
        }
        self.set_piece(t, s)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn try_place_piece(&mut self, t: Tile, c: PieceState) -> Result<(), Error> {
        if self.board.get_tile(t) == Some(PieceState::Empty) {
            self.set_piece(t, c)?;
            Ok(())
        } else {
            Err(Error::TileNotEmpty)
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            size: DEFAULT_SIZE,
            board: Board::new(DEFAULT_SIZE),
            to_play: Colour::Black,
            groups: Groups([UnionFind::new(0), UnionFind::new(0)]),
        }
    }
}

struct Groups([UnionFind; 2]);

impl Groups {
    pub fn get_mut(&mut self, c: Colour) -> &mut UnionFind {
        &mut self.0[c.group_idx()]
    }

    pub fn join(&mut self, t: Tile, c: Colour, board: &Board) {
        if t.edge(c) == 0 {
            self.0[c.group_idx()].union(Tile::Edge1, t);
        }
        if t.edge(c) == board.size - 1 {
            self.0[c.group_idx()].union(t, Tile::Edge2);
        }

        for n in board.neighbours(t) {
            match n {
                Some((neighbour, PieceState::Colour(n_colour))) if n_colour == c => {
                    self.0[c.group_idx()].union(neighbour, t);
                }
                Some(_) | None => {}
            }
        }
    }
}
