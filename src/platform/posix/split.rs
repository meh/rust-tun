use std::io::{self, Read, Write};
use std::sync::Arc;
use std::os::unix::io::{RawFd, AsRawFd};

use libc;
use platform::posix::Fd;

/// Read-only end for a file descriptor.
pub struct Reader(pub(crate) Arc<Fd>);

/// Write-only end for a file descriptor.
pub struct Writer(pub(crate) Arc<Fd>);

impl Read for Reader {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		unsafe {
			let amount = libc::read(self.0.as_raw_fd(), buf.as_mut_ptr() as *mut _, buf.len());

			if amount < 0 {
				return Err(io::Error::last_os_error().into());
			}

			Ok(amount as usize)
		}
	}
}

impl Write for Writer {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		unsafe {
			let amount = libc::write(self.0.as_raw_fd(), buf.as_ptr() as *const _, buf.len());

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

impl AsRawFd for Reader {
	fn as_raw_fd(&self) -> RawFd {
		self.0.as_raw_fd()
	}
}

impl AsRawFd for Writer {
	fn as_raw_fd(&self) -> RawFd {
		self.0.as_raw_fd()
	}
}
