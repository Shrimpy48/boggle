use crate::*;
use futures::io::{AsyncBufReadExt, AsyncWriteExt};
use futures::{StreamExt, TryStreamExt};
use gio::prelude::*;
use gtk::glib;
use std::convert::TryFrom;
use std::error;
use std::fmt;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    GLibError(glib::Error),
    SimpleMessage(&'static str),
    StringMessage(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(e) => e.fmt(f),
            Error::GLibError(e) => e.fmt(f),
            Error::SimpleMessage(msg) => f.pad(msg),
            Error::StringMessage(msg) => f.pad(msg),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IOError(e) => Some(e),
            Self::GLibError(e) => Some(e),
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

impl From<glib::Error> for Error {
    fn from(e: glib::Error) -> Self {
        Error::GLibError(e)
    }
}

pub async fn read_dice<P: AsRef<Path>>(path: P) -> Result<Dice, Error> {
    let f = gio::File::for_path(path)
        .read_async_future(glib::PRIORITY_DEFAULT)
        .await?;
    let buf_reader = f.into_async_buf_read(64);
    let dice_vec = buf_reader
        .lines()
        .map(|line_res| {
            let parsed = line_res?
                .parse::<BString>()
                .map_err(|_| Error::SimpleMessage("invalid character"))?;
            let die_vec = parsed.to_vec();
            <[BChar; 6]>::try_from(die_vec)
                .map_err(|_vec| Error::SimpleMessage("wrong number of faces"))
        })
        .try_collect::<Vec<[BChar; 6]>>()
        .await?;
    Dice::try_from(dice_vec).map_err(|_vec| Error::SimpleMessage("wrong number of dice"))
}

pub async fn try_read_dict<P: AsRef<Path>>(path: P) -> Result<Dict, Error> {
    let f = gio::File::for_path(path)
        .read_async_future(glib::PRIORITY_DEFAULT)
        .await?;
    let buf_reader = f.into_async_buf_read(64);
    buf_reader
        .lines()
        .map(|line_res| {
            let line = line_res?;
            if line.len() < 3 {
                return Err(Error::StringMessage(format!("word too short: {}", line)));
            }
            line.parse::<BString>()
                .map_err(|_| Error::StringMessage(format!("invalid word: {}", line)))
        })
        .try_collect::<Dict>()
        .await
}

pub async fn read_dict<P: AsRef<Path>>(path: P) -> Result<Dict, Error> {
    let f = gio::File::for_path(path)
        .read_async_future(glib::PRIORITY_DEFAULT)
        .await?;
    let buf_reader = f.into_async_buf_read(64);
    buf_reader
        .lines()
        .map_err(|e| e.into())
        .try_filter_map(|line| async move {
            if line.len() < 3 {
                return Ok(None);
            }
            Ok(line.parse::<BString>().ok())
        })
        .try_collect::<Dict>()
        .await
}

pub async fn write_dict<P: AsRef<Path>>(path: P, dict: Dict) -> Result<(), Error> {
    let f = gio::File::for_path(path)
        .create_readwrite_async_future(
            gio::FileCreateFlags::REPLACE_DESTINATION,
            glib::PRIORITY_DEFAULT,
        )
        .await?;
    let mut buf_writer = match f.into_async_read_write() {
        Ok(stream) => stream,
        Err(_) => return Err(Error::SimpleMessage("could not get async writer")),
    };
    for word in dict.words() {
        buf_writer.write_all(word.to_string().as_bytes()).await?;
    }
    buf_writer.flush().await.map_err(|e| e.into())
}
