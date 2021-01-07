use std::error;
use std::fmt;
use std::result;
use std::io;

use reqwest;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Reqwest(reqwest::Error),
    Scraping,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Reqwest(err) => write!(f, "Web request error: {}", err),
            Self::Scraping => f.write_str("Error scraping webpage"),
        }
    }
}

impl error::Error for Error {}

pub type Result<T> = result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}
