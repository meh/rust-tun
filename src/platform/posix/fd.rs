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

use std::io::{self, Read, Write};
use std::os::unix::io::{RawFd, AsRawFd};

use libc;
use error::Error;

/// POSIX file descriptor support for `io` traits and optionally for `mio`.
pub struct Fd(pub RawFd);

impl Fd {
	pub fn new(value: RawFd) -> Result<Self, Error> {
		if value < 0 {
			return Err(Error::InvalidDescriptor);
		}

		Ok(Fd(value))
	}
}

impl Read for Fd {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		unsafe {
			let amount = libc::read(self.0, buf.as_mut_ptr() as *mut _, buf.len());

			if amount < 0 {
				return Err(io::Error::last_os_error().into());
			}

			Ok(amount as usize)
		}
	}
}

impl Write for Fd {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		unsafe {
			let amount = libc::write(self.0, buf.as_ptr() as *const _, buf.len());

			if amount < 0 {
				return Err(io::Error::last_os_error().into());
			}

			Ok(amount as usize)
		}
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

impl AsRawFd for Fd {
	fn as_raw_fd(&self) -> RawFd {
		self.0
	}
}

impl Drop for Fd {
	fn drop(&mut self) {
		unsafe {
			libc::close(self.0);
		}
	}
}

#[cfg(feature = "mio")]
mod mio {
	use std::io;
	use mio::{Ready, Poll, PollOpt, Token};
	use mio::event::Evented;
	use mio::unix::EventedFd;

	impl Evented for super::Fd {
		fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
			EventedFd(&self.0).register(poll, token, interest, opts)
		}

		fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
			EventedFd(&self.0).reregister(poll, token, interest, opts)
		}

		fn deregister(&self, poll: &Poll) -> io::Result<()> {
			EventedFd(&self.0).deregister(poll)
		}
	}
}
