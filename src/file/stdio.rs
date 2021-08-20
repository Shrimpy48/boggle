use crate::*;
use std::convert::TryFrom;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    SimpleMessage(&'static str),
    StringMessage(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(e) => e.fmt(f),
            Error::SimpleMessage(msg) => f.pad(msg),
            Error::StringMessage(msg) => f.pad(msg),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IOError(e) => Some(e),
            Self::SimpleMessage(_) => None,
            Self::StringMessage(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IOError(e)
    }
}

pub fn read_dice<P: AsRef<Path>>(path: P) -> Result<Dice, Error> {
    let f = File::open(path)?;
    let buf_reader = io::BufReader::new(f);
    let mut dice_vec = Vec::new();
    for line_res in buf_reader.lines() {
        let parsed = line_res?
            .parse::<BString>()
            .map_err(|_| Error::SimpleMessage("invalid character"))?;
        let die_vec = parsed.to_vec();
        let die = <[BChar; 6]>::try_from(die_vec)
            .map_err(|_vec| Error::SimpleMessage("wrong number of faces"))?;
        dice_vec.push(die);
    }
    Dice::try_from(dice_vec).map_err(|_vec| Error::SimpleMessage("wrong number of dice"))
}

pub fn try_read_dict<P: AsRef<Path>>(path: P) -> Result<Dict, Error> {
    let f = File::open(path)?;
    let buf_reader = io::BufReader::new(f);
    let mut dict = Dict::default();
    for line_res in buf_reader.lines() {
        let line = line_res?;
        if line.len() < 3 {
            return Err(Error::StringMessage(format!("word too short: {}", line)));
        }
        let parsed = line
            .parse::<BString>()
            .map_err(|_| Error::StringMessage(format!("invalid word: {}", line)))?;
        dict.insert(&parsed);
    }
    Ok(dict)
}

pub fn read_dict<P: AsRef<Path>>(path: P) -> Result<Dict, Error> {
    let f = File::open(path)?;
    let buf_reader = io::BufReader::new(f);
    let mut dict = Dict::default();
    for line_res in buf_reader.lines() {
        let line = line_res?;
        if line.len() < 3 {
            continue;
        }
        if let Ok(parsed) = line.parse::<BString>() {
            dict.insert(&parsed);
        }
    }
    Ok(dict)
}

pub fn write_dict<P: AsRef<Path>>(path: P, dict: &Dict) -> Result<(), Error> {
    let f = File::create(path)?;
    let mut buf_writer = io::BufWriter::new(f);
    for word in dict.words() {
        writeln!(buf_writer, "{}", word)?;
    }
    buf_writer.flush().map_err(|e| e.into())
}
