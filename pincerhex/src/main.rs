use rustyline::{self, error::ReadlineError, Editor};

mod board;
mod tile;
mod union_find;

const HISTFILE: &str = "history.txt";

enum REPLError {
    InvalidCommand,
}

impl std::fmt::Display for REPLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            REPLError::InvalidCommand => f.write_str("invalid command"),
        }
    }
}

enum HexBotOutput {
    Empty,
}

impl std::fmt::Display for HexBotOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("todo")
    }
}

fn process_command(command: &str, args: &[&str]) -> Result<HexBotOutput, REPLError> {
    match command {
        "i" | "init_board" => {
            // init board here
            todo!("init board")
        }
        "b" | "show_board" => {
            // show board here
            todo!("show board")
        }
        "p" | "pretty_board" => {
            // show pretty board here
            todo!("pretty_board")
        }
        "v" | "make_move" => {
            // make move here
            todo!("make move");
        }
        "o" | "seto" => {
            // set opponent move here
            todo!("set opponent move here");
        }
        "y" | "sety" => {
            // set bot move here
            todo!("set bot move here")
        }
        "unset" => {
            // unset here
            todo!("unset move here")
        }
        "swap" => {
            // swap here
            todo!("swap move here")
        }
        "c" | "check_win" => {
            // check win here
            todo!("check win here")
        }
        "m" | "mcts" => {
            // set mcts params here
            todo!("set mcts params here")
        }
        "help_mcts" => {
            // display mcts params here
            todo!("display mcts params here")
        }
        &_ => Err(REPLError::InvalidCommand),
    }
}

fn process_line(line: &str) -> Result<HexBotOutput, REPLError> {
    let arr = line
        .split(' ')
        .filter_map(|s| match s.trim() {
            s if !s.is_empty() => Some(s),
            _ => None,
        })
        .collect::<Vec<&str>>();
    arr.first().map_or(Ok(HexBotOutput::Empty), |command| {
        process_command(command, &arr.as_slice()[1..])
    })
}

fn main() -> rustyline::Result<()> {
    let mut rl = Editor::<()>::new()?;
    if rl.load_history(HISTFILE).is_err() {
        eprintln!("No previous history.");
    }
    loop {
        let readline = rl.readline("");
        match readline {
            Ok(line) => {
                if line == "" {
                    continue;
                }
                rl.add_history_entry(line.as_str());
                match process_line(&line) {
                    Ok(out) => println!("{}", out),
                    Err(err) => eprintln!("{}", err),
                }
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                eprintln!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }
    rl.save_history(HISTFILE)
}
