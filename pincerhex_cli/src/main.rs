#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::implicit_return, clippy::question_mark_used)]

use rustyline::{self, error::ReadlineError, Editor};

use pincerhex_core::{
    BoardError, BotError, Colour, HexBot, Move, PieceState, StateError, TileError, Winner,
    STARTING_COLOUR,
};

#[allow(dead_code)]
const HISTFILE: &str = "history.txt";

enum REPLError {
    InvalidCommand,
    Usage(Usage),
    Bot(BotError),
}

enum Usage {
    InitBoard,
}

impl core::fmt::Display for Usage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InitBoard => write!(f, "usage: init_board <size>"),
        }
    }
}

impl core::fmt::Display for REPLError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidCommand => write!(f, "invalid command"),
            Self::Usage(u) => write!(f, "{u}"),
            Self::Bot(b) => match b {
                BotError::State(e) => match e {
                    StateError::TileNotEmpty => write!(f, "tile not empty"),
                    StateError::InvalidTile => write!(f, "invalid tile"),
                    StateError::Board(b) => match b {
                        BoardError::NotInRange => write!(f, "not in range"),
                    },
                },
                BotError::InvalidMove(m) => match m {
                    TileError::InvalidCol => write!(f, "invalid col"),
                    TileError::InvalidRow => write!(f, "invalid row"),
                },
                BotError::EmptyMove => write!(f, "empty move"),
            },
        }
    }
}

enum HexBotOutput {
    Empty,
    Move(Move),
    CheckWin(Option<Winner>),
    String(String),
}

impl core::fmt::Display for HexBotOutput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => Ok(()),
            Self::Move(m) => match m {
                Move::Move(mv) => write!(f, "{mv}"),
                Move::Swap => write!(f, "swap"),
            },
            Self::CheckWin(w) => match w {
                Some(Winner::Bot) => write!(f, "1"),
                Some(Winner::Opponent) => write!(f, "-1"),
                None => write!(f, "0"),
            },
            Self::String(s) => write!(f, "{s}"),
        }
    }
}

impl From<BotError> for REPLError {
    fn from(value: BotError) -> Self {
        Self::Bot(value)
    }
}

fn process_command(
    bot: &mut HexBot,
    command: &str,
    args: &[&str],
) -> Result<HexBotOutput, REPLError> {
    match command {
        "i" | "init_board" => {
            let size = args
                .first()
                .and_then(|s| s.parse::<i8>().ok())
                .ok_or(REPLError::Usage(Usage::InitBoard))?;
            bot.init_board(size);
            Ok(HexBotOutput::Empty)
        }
        "b" | "show_board" => Ok(HexBotOutput::String(bot.get_compressed())),
        "p" | "pretty_board" => Ok(HexBotOutput::String(bot.get_pretty())),
        "v" | "make_move" => {
            let mv = bot.make_move()?;
            Ok(HexBotOutput::Move(mv))
        }
        "o" | "seto" => {
            bot.set_tile(args.first(), PieceState::Colour(bot.colour().opponent()))?;
            Ok(HexBotOutput::Empty)
        }
        "y" | "sety" => {
            bot.set_tile(args.first(), PieceState::Colour(bot.colour()))?;
            Ok(HexBotOutput::Empty)
        }
        "unset" => {
            bot.set_tile(args.first(), PieceState::Empty)?;
            Ok(HexBotOutput::Empty)
        }
        "swap" => {
            bot.swap();
            Ok(HexBotOutput::Empty)
        }
        "c" | "check_win" => Ok(HexBotOutput::CheckWin(bot.check_win())),
        &_ => Err(REPLError::InvalidCommand),
    }
}

fn process_line(bot: &mut HexBot, line: &str) -> Result<HexBotOutput, REPLError> {
    let arr = line
        .split(' ')
        .filter_map(|s| match s.trim() {
            s if !s.is_empty() => Some(s),
            _ => None,
        })
        .collect::<Vec<&str>>();
    arr.first().map_or(Ok(HexBotOutput::Empty), |command| {
        process_command(bot, command, &arr.as_slice()[1..])
    })
}

enum Error {
    Readline(rustyline::error::ReadlineError),
    Usage(String),
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Readline(err) => write!(f, "{err}"),
            Self::Usage(s) => write!(f, "usage: {s} <colour>"),
        }
    }
}

impl From<rustyline::error::ReadlineError> for Error {
    fn from(value: rustyline::error::ReadlineError) -> Self {
        Self::Readline(value)
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    let colour = args
        .get(1)
        .and_then(|s| Colour::try_from(s).ok())
        .ok_or(Error::Usage(args[0].clone()))?;
    unsafe {
        STARTING_COLOUR = colour;
    }

    let mut bot = HexBot::new(colour);
    let mut rl = Editor::<()>::new()?;
    loop {
        let readline = rl.readline("");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(line.as_str());
                match process_line(&mut bot, &line) {
                    Ok(HexBotOutput::Empty) => {}
                    Ok(out) => println!("{out}"),
                    Err(err) => eprintln!("{err}"),
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }
    Ok(())
}
