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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid configuration")]
    InvalidConfig,

    #[error("not implementated")]
    NotImplemented,

    #[error("device name too long")]
    NameTooLong,

    #[error("invalid device name")]
    InvalidName,

    #[error("invalid address")]
    InvalidAddress,

    #[error("invalid file descriptor")]
    InvalidDescriptor,

    #[error("unsuported network layer of operation")]
    UnsupportedLayer,

    #[error("invalid queues number")]
    InvalidQueuesNumber,

    #[error("out of range integral type conversion attempted")]
    TryFromIntError,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Nul(#[from] std::ffi::NulError),

    #[error(transparent)]
    ParseNum(#[from] std::num::ParseIntError),

    #[cfg(target_os = "windows")]
    #[error(transparent)]
    WintunError(#[from] wintun::Error),
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Io(err) => err,
            _ => std::io::Error::new(std::io::ErrorKind::Other, value),
        }
    }
}

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T, E = Error> = ::std::result::Result<T, E>;
