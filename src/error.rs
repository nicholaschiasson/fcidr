use std::{error, fmt};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Error {
    InvalidNetwork(String),
    InvalidPrefix(String),
    Parse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for Error {}
