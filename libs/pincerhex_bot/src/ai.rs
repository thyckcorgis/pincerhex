use crate::StdRng;
use alloc::string::String;
use pincerhex_core::{first_move, Colour, Move, PieceState, PotentialEvaluator, Tile, TileError};
use pincerhex_state::{Error as StateError, State, Winner, DEFAULT_SIZE};

/// Whether or not to play with the swap rule
/// Should probably be an environment variable
pub const SWAP_RULE: bool = true;

#[derive(Debug, Clone, Copy)]
pub enum SwapRole {
    Start,
    Swap,
}

impl From<Colour> for SwapRole {
    fn from(value: Colour) -> Self {
        match value {
            Colour::Black => Self::Start,
            Colour::White => Self::Swap,
        }
    }
}

pub struct HexBot {
    colour: Colour,
    starting: Colour,
    state: State,
    size: i8,
    #[allow(dead_code)]
    allow_invalid: bool,
    swap_state: Option<SwapRole>,
    move_count: u16,
}

#[derive(Debug)]
pub enum BotError {
    State(StateError),
    EmptyMove,
    InvalidMove(TileError),
}

impl core::fmt::Display for BotError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::State(e) => write!(f, "{e}"),
            Self::InvalidMove(m) => write!(f, "{m}"),
            Self::EmptyMove => write!(f, "empty move"),
        }
    }
}

impl From<StateError> for BotError {
    fn from(v: StateError) -> Self {
        Self::State(v)
    }
}

impl HexBot {
    #[must_use]
    pub fn new(c: Colour) -> Self {
        Self {
            colour: c,
            starting: c,
            state: State::default(),
            size: DEFAULT_SIZE,
            allow_invalid: true,
            swap_state: Some(SwapRole::from(c)),
            move_count: 0,
        }
    }

    #[must_use]
    pub const fn colour(&self) -> Colour {
        self.colour
    }

    fn place_piece(&mut self, mv: Tile, state: PieceState) -> Result<(), BotError> {
        self.state.place_piece(mv, state).map_err(BotError::State)
    }

    /// # Errors
    /// Will return `Err` if given an invalid or empty move
    pub fn set_tile(&mut self, mv: Option<&&str>, state: PieceState) -> Result<(), BotError> {
        mv.ok_or(BotError::EmptyMove)
            .and_then(|s| Tile::try_from(*s).map_err(BotError::InvalidMove))
            .and_then(|mv| self.place_piece(mv, state))
    }

    pub fn init_board(&mut self, size: i8) {
        self.state = State::new(size);
        self.size = size;
        self.swap_state = Some(SwapRole::from(self.colour));
        self.move_count = 0;
    }

    #[must_use]
    pub fn get_compressed(&self) -> String {
        self.state.get_compressed()
    }

    #[must_use]
    pub fn get_pretty(&self) -> String {
        self.state.get_pretty()
    }

    /// # Errors
    /// Will return an `Err` if applying the swap rule failed
    pub fn make_move(&mut self) -> Result<Move, BotError> {
        if let (Some(s), true) = (self.swap_state, SWAP_RULE) {
            let mv = self.handle_swap(s)?;
            self.swap_state = None;
            Ok(mv)
        } else {
            Ok(Move::Move(self.regular_move()))
        }
    }

    fn handle_swap(&mut self, s: SwapRole) -> Result<Move, BotError> {
        let mut rng = StdRng(rand::thread_rng());
        match s {
            SwapRole::Start => {
                let (i, j) = first_move(self.size, &mut rng);
                let mv = Tile::Regular(i, j);
                self.state
                    .place_piece(mv, PieceState::Colour(self.colour))?;
                self.move_count += 1;
                Ok(Move::Move(mv))
            }
            SwapRole::Swap => {
                if self.state.should_swap(&mut rng) {
                    self.swap();
                    Ok(Move::Swap)
                } else {
                    Ok(Move::Move(self.regular_move()))
                }
            }
        }
    }

    fn regular_move(&mut self) -> Tile {
        let mut rng = StdRng(rand::thread_rng());
        let (i, j) = PotentialEvaluator::new(self.state.get_board(), self.colour, self.starting)
            .evaluate()
            .get_best_move(self.move_count, &mut rng);
        let mv = Tile::Regular(i, j);

        self.place_piece(mv, PieceState::Colour(self.colour))
            .expect("valid move");
        self.move_count += 1;
        mv
    }

    pub fn check_win(&mut self) -> Option<Winner> {
        self.state.get_winner(self.colour)
    }

    pub fn swap(&mut self) {
        self.colour = self.colour.opponent();
    }
}
