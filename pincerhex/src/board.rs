// Totally not stolen from central_program

use std::collections::HashSet;

use crate::tile::{Colour, PieceState, Tile};

#[derive(Debug)]
pub struct Board {
    pub size: i8,
    board: Vec<PieceState>,
}

#[derive(Debug)]
pub enum Error {
    NotInRange,
}

impl<'a> Board {
    pub fn new(size: i8) -> Self {
        Self {
            size,
            board: vec![PieceState::Empty; (size as usize).pow(2)],
        }
    }

    pub fn get(&self, r: i8, c: i8) -> Option<PieceState> {
        self.get_tile(Tile::Valid(r, c))
    }

    #[allow(dead_code)]
    pub fn swap_pieces(&mut self) -> Result<(), Error> {
        let mut to_swap: HashSet<Tile> = HashSet::new();

        for tile in self.iter() {
            if let (Tile::Valid(row, col), PieceState::Colour(_)) = (tile.0, tile.1) {
                if !to_swap.contains(&Tile::Valid(col, row)) {
                    to_swap.insert(tile.0);
                }
            }
        }

        for &tile in &to_swap {
            if let Tile::Valid(row, col) = tile {
                let other = Tile::Valid(col, row);
                let (i, j) = (self.get_tile(tile), self.get_tile(other));
                if let Some(c) = i {
                    let piece = match c {
                        PieceState::Colour(c) => PieceState::Colour(c.opponent()),
                        PieceState::Empty => PieceState::Empty,
                    };
                    self.set_tile(other, piece)?;
                }
                if let Some(c) = j {
                    let piece = match c {
                        PieceState::Colour(c) => PieceState::Colour(c.opponent()),
                        PieceState::Empty => PieceState::Empty,
                    };
                    self.set_tile(tile, piece)?;
                }
            }
        }
        Ok(())
    }

    pub fn get_compressed(&self) -> String {
        let mut chars: Vec<char> = self
            .board
            .iter()
            .map(|tile| match tile {
                PieceState::Colour(Colour::Black) => 'B',
                PieceState::Colour(Colour::White) => 'W',
                PieceState::Empty => '.',
            })
            .collect();

        for i in (1..=self.size as usize).rev() {
            chars.insert(i * (self.size as usize), '|');
        }

        chars.into_iter().collect()
    }

    pub fn neighbour(&self, tile: Tile, row: i8, col: i8) -> Option<(Tile, PieceState)> {
        let n = tile.neighbour(row, col);
        n.to_index(self.size)
            .map(|idx| self.board[idx])
            .map(|state| (n, state))
    }

    pub const fn iter(&'a self) -> Iter<'a> {
        Iter {
            idx: 0,
            board: self,
        }
    }

    pub fn index_to_tile(&self, idx: usize) -> Tile {
        if idx >= self.board.len() {
            Tile::Invalid
        } else {
            Tile::Valid(
                (idx / self.size as usize) as i8,
                (idx % self.size as usize) as i8,
            )
        }
    }

    pub fn neighbours(&self, tile: Tile) -> [Option<(Tile, PieceState)>; 6] {
        [
            self.neighbour(tile, 0, 1),
            self.neighbour(tile, 1, 0),
            self.neighbour(tile, 1, -1),
            self.neighbour(tile, 0, -1),
            self.neighbour(tile, -1, 0),
            self.neighbour(tile, -1, 1),
        ]
    }

    pub fn get_tile(&self, tile: Tile) -> Option<PieceState> {
        tile.to_index(self.size)
            .and_then(|idx| self.board.get(idx).copied())
    }

    pub fn set_tile(&mut self, tile: Tile, s: PieceState) -> Result<(), Error> {
        let idx = tile.to_index(self.size).ok_or(Error::NotInRange)?;
        self.board[idx] = s;
        Ok(())
    }
}

impl std::fmt::Display for Board {
    /// Example output:
    /// B . . .
    ///  . B W .
    ///   . . B .
    ///    W . W B
    /// ------------------
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..(self.size as usize) {
            write!(f, "{}", " ".repeat(r))?;

            for c in 0..(self.size as usize) {
                if let Some(c) = self.get_tile(Tile::Valid(r as i8, c as i8)) {
                    match c {
                        PieceState::Colour(Colour::Black) => write!(f, "B ")?,
                        PieceState::Colour(Colour::White) => write!(f, "W ")?,
                        PieceState::Empty => write!(f, ". ")?,
                    }
                }
            }
            writeln!(f)?;
        }

        write!(f, "{}", "-".repeat(18))
    }
}

pub struct Iter<'a> {
    idx: isize,
    board: &'a Board,
}

impl<'a> Iter<'a> {
    fn get(&self) -> (Tile, PieceState) {
        let state = self
            .board
            .board
            .get(self.idx as usize)
            .copied()
            // To panic or not to panic...
            .unwrap_or(PieceState::Empty);
        let tile = self.board.index_to_tile(self.idx as usize);
        (tile, state)
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.idx <= -1 {
            return None;
        }
        let (tile, state) = self.get();
        self.idx -= 1;
        Some((tile, state))
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Tile, PieceState);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx as usize >= self.board.board.len() {
            return None;
        }
        let (tile, state) = self.get();
        self.idx += 1;
        Some((tile, state))
    }
}

#[cfg(test)]
mod board_testing {
    use super::{Board, Colour, PieceState, Tile};

    macro_rules! set {
        ($board: ident, $row: expr, $col: expr, $colour: expr) => {
            let res = $board.set_tile(Tile::Valid($row, $col), PieceState::Colour($colour));
            assert!(matches!(res, Ok(())))
        };
    }

    #[test]
    fn drawing() {
        let mut board = Board::new(4);

        set!(board, 0, 0, Colour::Black);
        set!(board, 1, 1, Colour::Black);
        set!(board, 2, 2, Colour::Black);
        set!(board, 3, 3, Colour::Black);

        set!(board, 3, 0, Colour::White);
        set!(board, 3, 2, Colour::White);
        set!(board, 1, 2, Colour::White);

        let expected = "B . . . \n . B W . \n  . . B . \n   W . W B \n------------------";

        assert_eq!(format!("{board}"), expected);

        set!(board, 0, 1, Colour::Black);
        set!(board, 0, 2, Colour::Black);
        set!(board, 0, 3, Colour::Black);

        let expected2 = "B B B B \n . B W . \n  . . B . \n   W . W B \n------------------";

        assert_eq!(format!("{board}"), expected2);
    }
}
