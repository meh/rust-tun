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

use crate::error::{Error, Result};
use libc::{self, F_GETFL, F_SETFL, O_NONBLOCK, fcntl};
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

/// POSIX file descriptor support for `io` traits.
pub(crate) struct Fd {
    pub(crate) inner: RawFd,
    close_fd_on_drop: bool,
}

impl Fd {
    pub fn new(value: RawFd, close_fd_on_drop: bool) -> Result<Self> {
        if value < 0 {
            return Err(Error::InvalidDescriptor);
        }
        Ok(Fd {
            inner: value,
            close_fd_on_drop,
        })
    }

    /// Enable non-blocking mode
    pub(crate) fn set_nonblock(&self) -> std::io::Result<()> {
        match unsafe { fcntl(self.inner, F_SETFL, fcntl(self.inner, F_GETFL) | O_NONBLOCK) } {
            0 => Ok(()),
            _ => Err(std::io::Error::last_os_error()),
        }
    }

    /// Wait until the file descriptor becomes readable or `timeout` expires.
    ///
    /// Returns an error of kind `std::io::ErrorKind::TimedOut` if the
    /// descriptor does not become readable within `timeout`. A zero
    /// `timeout` checks readability without blocking. If poll(2) reports an
    /// error or hangup condition instead of readability, this returns
    /// `Ok(())` and lets the following read report the actual condition.
    pub(crate) fn wait_readable(&self, timeout: std::time::Duration) -> std::io::Result<()> {
        let deadline = std::time::Instant::now().checked_add(timeout);
        loop {
            // Round the remaining time up to whole milliseconds so a
            // fractional remainder does not turn into a zero-timeout poll
            // before the deadline is actually reached.
            let millis = match deadline {
                Some(deadline) => {
                    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                    libc::c_int::try_from(remaining.as_nanos().div_ceil(1_000_000))
                        .unwrap_or(libc::c_int::MAX)
                }
                // `timeout` overflows `Instant`; wait as long as poll(2)
                // allows per call and keep re-arming.
                None => libc::c_int::MAX,
            };
            let mut pollfd = libc::pollfd {
                fd: self.inner,
                events: libc::POLLIN,
                revents: 0,
            };
            match unsafe { libc::poll(&mut pollfd, 1, millis) } {
                -1 => {
                    let err = std::io::Error::last_os_error();
                    if err.kind() != std::io::ErrorKind::Interrupted {
                        return Err(err);
                    }
                    // Interrupted by a signal; re-arm with the remaining time.
                }
                0 => {
                    if deadline.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
                        return Err(std::io::ErrorKind::TimedOut.into());
                    }
                    // The wait was clamped below the remaining time; re-arm.
                }
                _ => return Ok(()),
            }
        }
    }

    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        let fd = self.as_raw_fd();
        let amount = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        if amount < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(amount as usize)
    }

    #[allow(dead_code)]
    #[inline]
    fn readv(&self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        if bufs.len() > max_iov() {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        }
        let amount = unsafe {
            libc::readv(
                self.as_raw_fd(),
                bufs.as_mut_ptr() as *mut libc::iovec as *const libc::iovec,
                bufs.len() as libc::c_int,
            )
        };
        if amount < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(amount as usize)
    }

    #[inline]
    pub fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        let fd = self.as_raw_fd();
        let amount = unsafe { libc::write(fd, buf.as_ptr() as *const _, buf.len()) };
        if amount < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(amount as usize)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn writev(&self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        if bufs.len() > max_iov() {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        }
        let amount = unsafe {
            libc::writev(
                self.as_raw_fd(),
                bufs.as_ptr() as *const libc::iovec,
                bufs.len() as libc::c_int,
            )
        };
        if amount < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(amount as usize)
    }
}

impl AsRawFd for Fd {
    fn as_raw_fd(&self) -> RawFd {
        self.inner
    }
}

impl IntoRawFd for Fd {
    fn into_raw_fd(mut self) -> RawFd {
        let fd = self.inner;
        self.inner = -1;
        fd
    }
}

impl Drop for Fd {
    fn drop(&mut self) {
        if self.close_fd_on_drop && self.inner >= 0 {
            unsafe { libc::close(self.inner) };
        }
    }
}

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_vendor = "apple",
))]
pub(crate) const fn max_iov() -> usize {
    libc::IOV_MAX as usize
}

#[cfg(any(
    target_os = "android",
    target_os = "emscripten",
    target_os = "linux",
    target_os = "nto",
))]
pub(crate) const fn max_iov() -> usize {
    libc::UIO_MAXIOV as usize
}

#[cfg(test)]
mod test {
    use super::Fd;
    use std::time::{Duration, Instant};

    fn pipe() -> (Fd, Fd) {
        let mut fds = [0; 2];
        let res = unsafe { libc::pipe(fds.as_mut_ptr()) };
        assert_eq!(
            res,
            0,
            "pipe(2) failed: {}",
            std::io::Error::last_os_error()
        );
        (
            Fd::new(fds[0], true).unwrap(),
            Fd::new(fds[1], true).unwrap(),
        )
    }

    #[test]
    fn wait_readable_times_out_on_a_silent_fd() {
        let (rx, _tx) = pipe();
        let timeout = Duration::from_millis(100);
        let start = Instant::now();
        let err = rx.wait_readable(timeout).unwrap_err();
        let elapsed = start.elapsed();
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
        assert!(elapsed >= timeout, "returned after {elapsed:?}");
        assert!(elapsed < Duration::from_secs(10), "took {elapsed:?}");
    }

    #[test]
    fn wait_readable_sees_data_already_present() {
        let (rx, tx) = pipe();
        tx.write(b"x").unwrap();
        rx.wait_readable(Duration::from_secs(5)).unwrap();
    }

    #[test]
    fn wait_readable_wakes_when_data_arrives_before_expiry() {
        let (rx, tx) = pipe();
        let writer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            tx.write(b"x").unwrap();
        });
        let start = Instant::now();
        rx.wait_readable(Duration::from_secs(10)).unwrap();
        assert!(start.elapsed() < Duration::from_secs(10));
        writer.join().unwrap();
    }

    #[test]
    fn wait_readable_zero_timeout_does_not_block() {
        let (rx, _tx) = pipe();
        let err = rx.wait_readable(Duration::ZERO).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    }
}
