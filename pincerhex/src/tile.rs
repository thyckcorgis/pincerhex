use core::str::FromStr;
use std::fmt::Write;

// Black goes top -> bottom. White goes left -> right
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Colour {
    Black,
    Empty,
    White,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Tile {
    Tile(u8, u8),
    Edge1,
    Edge2,
    Invalid,
}

pub enum TileError {
    InvalidCol,
    InvalidRow,
}

impl FromStr for Tile {
    type Err = TileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = s.chars().nth(0).ok_or(TileError::InvalidRow)? as u8 - 97; // First letter
        let c = s[1..].parse::<u8>().map_err(|_| TileError::InvalidCol)? - 1; // All following digits

        Ok(Tile::Tile(r, c))
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Tile::Tile(row, col) => {
                f.write_char((row + 97) as char)?;
                f.write_fmt(format_args!("{}", col + 1))
            }
            Tile::Edge1 => f.write_str("edge1"),
            Tile::Edge2 => f.write_str("edge2"),
            Tile::Invalid => f.write_str("invalid"),
        }
    }
}

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Black => write!(f, "Black"),
            Self::White => write!(f, "White"),
            Self::Empty => write!(f, "Empty"),
        }
    }
}
