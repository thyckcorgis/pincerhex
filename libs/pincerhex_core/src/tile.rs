use core::fmt::Write;

use alloc::string::String;

// Black goes top -> bottom. White goes left -> right
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Colour {
    Black,
    White,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceState {
    Colour(Colour),
    Empty,
}

impl Colour {
    #[must_use]
    pub const fn group_idx(self) -> usize {
        match self {
            Self::Black => 0,
            Self::White => 1,
        }
    }

    #[must_use]
    pub const fn opponent(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash, Ord)]
pub enum Tile {
    Regular(i8, i8),
    Edge1,
    Edge2,
    Invalid,
}

impl Tile {
    /// # Panics
    /// Panics if called on an invalid tile
    #[must_use]
    pub const fn edge(self, colour: Colour) -> i8 {
        match (self, colour) {
            (Self::Regular(r, _), Colour::Black) => r,
            (Self::Regular(_, c), Colour::White) => c,
            (_, _) => panic!("called edge on an invalid tile"),
        }
    }

    #[must_use]
    pub const fn to_index(self, size: i8) -> Option<usize> {
        match self {
            Self::Regular(r, c) if r >= 0 && r < size && c >= 0 && c < size => {
                Some((r as usize) * size as usize + c as usize)
            }
            Self::Regular(_, _) | Self::Edge1 | Self::Edge2 | Self::Invalid => None,
        }
    }

    #[must_use]
    pub const fn neighbour(self, row: i8, col: i8) -> Self {
        match self {
            Self::Regular(r, c) => Self::Regular(r + row, c + col),
            Self::Edge1 | Self::Edge2 | Self::Invalid => Self::Invalid,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidCol,
    InvalidRow,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidCol => write!(f, "invalid col"),
            Self::InvalidRow => write!(f, "invalid row"),
        }
    }
}

impl TryFrom<&str> for Tile {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let r = s.chars().next().ok_or(Error::InvalidRow)? as i8 - 97; // First letter
        let c = s[1..].parse::<i8>().map_err(|_| Error::InvalidCol)? - 1; // All following digits
        Ok(Self::Regular(r, c))
    }
}

impl core::fmt::Display for Tile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self {
            Self::Regular(row, col) => {
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
