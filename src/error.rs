//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.

use std::{ffi, io, num};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Name too long")]
    NameTooLong,

    #[fail(display = "Invalid name")]
    InvalidName,

    #[fail(display = "Invalid address")]
    InvalidAddress,

    #[fail(display = "Invalid descriptor")]
    InvalidDescriptor,

    #[fail(display = "IO error: {}", error)]
    Io { error: io::Error },

    #[fail(display = "FFI nul error: {}", error)]
    Nul { error: ffi::NulError },

    #[fail(display = "Integer parse error: {}", error)]
    ParseNum { error: num::ParseIntError },
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io { error }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(error: num::ParseIntError) -> Self {
        Error::ParseNum { error }
    }
}

impl From<ffi::NulError> for Error {
    fn from(error: ffi::NulError) -> Self {
        Error::Nul { error }
    }
}
