use std::{
    fs::{self, remove_dir, remove_file},
    io::{self, Write},
    path::Path,
    process::{exit, Command},
};
use tempfile::tempdir;

use std::os::unix::fs::OpenOptionsExt;

const HEXBOT: &[u8] = include_bytes!("../hexterminator");

#[derive(Debug)]
pub enum Error {
    NoExitCode,
    IoError(io::Error),
    CtrlCError(ctrlc::Error),
}

impl From<ctrlc::Error> for Error {
    fn from(value: ctrlc::Error) -> Self {
        Self::CtrlCError(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

fn write_file(path: &Path) -> Result<(), Error> {
    let mut file = fs::OpenOptions::new()
        .mode(755)
        .create(true)
        .write(true)
        .open(&path)?;

    file.write_all(HEXBOT)?;
    Ok(())
}

fn start_bot() -> Result<i32, Error> {
    let dir = tempdir()?;
    let path = dir.path().join("hexterminator");
    write_file(&path)?;
    let bot_path = path.clone();
    ctrlc::set_handler(move || {
        remove_file(&bot_path)
            .map(|_| bot_path.parent().map(|d| remove_dir(d)))
            // rust bby
            .unwrap()
            .unwrap()
            .unwrap();

        // hexterminator inherits stdin so it SHOULD exit automatically.
        // No need to handle here.
        exit(1);
    })?;
    let mut child = Command::new(&path).arg("white").spawn()?;
    let result = child.wait()?.code().ok_or(Error::NoExitCode)?;
    remove_file(&path)?;
    Ok(result)
}

pub fn hexterminate() -> Result<(), Error> {
    exit(start_bot()?)
}
