use std::fmt::Write;

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

    #[cfg(feature = "explore")]
    pub const fn val(self) -> i8 {
        match self {
            Self::Black => -1,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Black => write!(f, "Black"),
            Self::White => write!(f, "White"),
        }
    }
}
