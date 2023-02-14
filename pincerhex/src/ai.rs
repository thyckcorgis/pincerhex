use rand::Rng;
use std::time::Duration;

use crate::{
    state::{self, State, DEFAULT_SIZE},
    tile::{self, Colour, Move, PieceState, SwapRole, Tile},
    Winner,
};

pub struct HexBot {
    colour: Colour,
    state: State,
    size: u8,
    #[allow(dead_code)]
    allow_invalid: bool,
    swap_state: Option<SwapRole>,
    move_count: u32,
    #[allow(dead_code)]
    params: MCTSParams,
}

#[allow(dead_code)]
struct MCTSParams {
    rounds: f64,
    best: f64,
    exp: f64,
    timed: bool,
    timeout: Duration,
}

impl Default for MCTSParams {
    fn default() -> Self {
        Self {
            rounds: 35_000.,
            best: 1.,
            exp: 10.,
            timeout: Duration::from_secs(5),
            timed: false,
        }
    }
}

pub enum BotError {
    State(state::Error),
    EmptyMove,
    InvalidMove(tile::Error),
}

impl From<state::Error> for BotError {
    fn from(v: state::Error) -> Self {
        Self::State(v)
    }
}

impl HexBot {
    pub fn new(c: Colour) -> Self {
        Self {
            colour: c,
            state: State::default(),
            size: DEFAULT_SIZE,
            allow_invalid: true,
            swap_state: Some(SwapRole::from(c)),
            move_count: 0,
            params: MCTSParams::default(),
        }
    }

    pub const fn colour(&self) -> Colour {
        self.colour
    }

    pub fn set_tile(&mut self, mv: Option<&&str>, state: PieceState) -> Result<(), BotError> {
        self.state
            .place_piece(
                mv.ok_or(BotError::EmptyMove)
                    .and_then(|s| (*s).try_into().map_err(BotError::InvalidMove))?,
                state,
            )
            .map_err(BotError::State)
    }

    pub fn init_board(&mut self, size: u8) {
        self.state = State::new(size);
        self.size = size;
        self.move_count = 0;
    }

    pub fn get_compressed(&self) -> String {
        self.state.get_compressed()
    }

    pub fn get_pretty(&self) -> String {
        self.state.get_pretty()
    }

    pub fn make_move(&mut self) -> Result<Move, BotError> {
        match self.swap_state {
            Some(s) => {
                let mv = self.handle_swap(s)?;
                self.swap_state = None;
                self.move_count += 1;
                Ok(mv)
            }
            None => Ok(Move::Move(self.regular_move())),
        }
    }

    fn handle_swap(&mut self, s: SwapRole) -> Result<Move, BotError> {
        match s {
            SwapRole::Start => {
                let mut rng = rand::thread_rng();
                let (mut i, mut j) = (rng.gen_range(0..4), rng.gen_range(0..4));
                if rng.gen_range(0..2) == 0 {
                    i = self.size - 1 - i;
                    j = self.size - 1 - j;
                }
                let mv = Tile::Valid(i, j);
                self.state
                    .place_piece(mv, PieceState::Colour(self.colour))?;
                Ok(Move::Move(mv))
            }
            SwapRole::Swap => {
                if self.state.should_swap() {
                    self.state.swap_pieces()?;
                    Ok(Move::Swap)
                } else {
                    Ok(Move::Move(self.regular_move()))
                }
            }
        }
    }

    fn regular_move(&mut self) -> Tile {
        todo!("make regular move for {} bot", self.colour)
    }

    pub fn check_win(&mut self) -> Option<Winner> {
        self.state.get_winner(self.colour)
    }

    pub fn swap(&mut self) -> Result<(), BotError> {
        self.state.swap_pieces()?;
        Ok(())
    }
}
