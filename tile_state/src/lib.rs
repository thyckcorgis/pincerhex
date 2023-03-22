// Totally not stolen from central_program
extern crate alloc;

pub mod board {
    use crate::tile::Colour;

    use super::tile::{PieceState, Tile};
    use alloc::collections::BTreeSet;

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
            let mut to_swap: BTreeSet<Tile> = BTreeSet::new();

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

    impl core::fmt::Display for Board {
        /// Example output:
        /// B . . .
        ///  . B W .
        ///   . . B .
        ///    W . W B
        /// ------------------
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
}

pub mod tile {
    use core::fmt::Write;

    // Black goes top -> bottom. White goes left -> right
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    pub enum Colour {
        Black,
        White,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum SwapRole {
        Start,
        Swap,
    }

    pub enum Move {
        Move(Tile),
        Swap,
    }

    pub struct InvalidColour;

    impl TryFrom<&String> for Colour {
        type Error = InvalidColour;

        fn try_from(value: &String) -> Result<Self, Self::Error> {
            match value.to_lowercase().as_str() {
                "w" | "white" => Ok(Self::White),
                "b" | "black" => Ok(Self::Black),
                _ => Err(InvalidColour),
            }
        }
    }

    impl From<Colour> for SwapRole {
        fn from(value: Colour) -> Self {
            match value {
                Colour::Black => Self::Start,
                Colour::White => Self::Swap,
            }
        }
    }

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    pub enum PieceState {
        Colour(Colour),
        Empty,
    }

    impl Colour {
        pub const fn group_idx(self) -> usize {
            match self {
                Self::Black => 0,
                Self::White => 1,
            }
        }

        pub const fn opponent(self) -> Self {
            match self {
                Self::Black => Self::White,
                Self::White => Self::Black,
            }
        }
    }

    #[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash, Ord)]
    pub enum Tile {
        Valid(i8, i8),
        Edge1,
        Edge2,
        Invalid,
    }

    impl Tile {
        pub const fn edge(self, colour: Colour) -> i8 {
            match (self, colour) {
                (Self::Valid(r, _), Colour::Black) => r,
                (Self::Valid(_, c), Colour::White) => c,
                (_, _) => panic!("called edge on an invalid tile"),
            }
        }

        pub const fn to_index(self, size: i8) -> Option<usize> {
            match self {
                Self::Valid(r, c) if r >= 0 && r < size && c >= 0 && c < size => {
                    Some((r as usize) * size as usize + c as usize)
                }
                Self::Valid(_, _) | Self::Edge1 | Self::Edge2 | Self::Invalid => None,
            }
        }

        pub const fn neighbour(self, row: i8, col: i8) -> Self {
            match self {
                Self::Valid(r, c) => Self::Valid(r + row, c + col),
                Self::Edge1 | Self::Edge2 | Self::Invalid => Self::Invalid,
            }
        }
    }

    #[derive(Debug)]
    pub enum Error {
        InvalidCol,
        InvalidRow,
    }

    impl TryFrom<&str> for Tile {
        type Error = Error;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            let r = s.chars().next().ok_or(Error::InvalidRow)? as i8 - 97; // First letter
            let c = s[1..].parse::<i8>().map_err(|_| Error::InvalidCol)? - 1; // All following digits
            Ok(Self::Valid(r, c))
        }
    }

    impl core::fmt::Display for Tile {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match &self {
                Self::Valid(row, col) => {
                    f.write_char((row + 97) as u8 as char)?;
                    f.write_fmt(format_args!("{}", col + 1))
                }
                Self::Edge1 => f.write_str("edge1"),
                Self::Edge2 => f.write_str("edge2"),
                Self::Invalid => f.write_str("invalid"),
            }
        }
    }

    impl core::fmt::Display for Colour {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match &self {
                Self::Black => write!(f, "Black"),
                Self::White => write!(f, "White"),
            }
        }
    }
}
