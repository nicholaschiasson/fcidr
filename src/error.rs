use alloc::string::String;
use core::{error, fmt::Display};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Error {
    InvalidNetwork(String),
    InvalidPrefix(String),
    Parse(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for Error {}
