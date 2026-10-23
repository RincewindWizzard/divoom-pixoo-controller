//! Minimal RFCOMM client socket (Linux / BlueZ kernel sockets).
//!
//! Only talks to the kernel via `libc`, so no libbluetooth / D-Bus is needed.
//! The device must already be paired (see `docs/setup.md`).

use anyhow::{bail, Context, Result};
use std::io::{self, Read, Write};
use std::time::Duration;

const BTPROTO_RFCOMM: libc::c_int = 3;

#[repr(C)]
struct SockaddrRc {
    family: libc::sa_family_t,
    bdaddr: [u8; 6],
    channel: u8,
}

/// Parse `AA:BB:CC:DD:EE:FF` into the little-endian `bdaddr_t` BlueZ expects.
pub fn parse_bdaddr(addr: &str) -> Result<[u8; 6]> {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() != 6 {
        bail!("invalid Bluetooth address '{addr}', expected AA:BB:CC:DD:EE:FF");
    }
    let mut out = [0u8; 6];
    for (i, p) in parts.iter().enumerate() {
        out[5 - i] = u8::from_str_radix(p, 16)
            .with_context(|| format!("invalid Bluetooth address '{addr}'"))?;
    }
    Ok(out)
}

pub struct RfcommStream {
    fd: libc::c_int,
}

impl RfcommStream {
    pub fn connect(addr: &str, channel: u8, timeout: Duration) -> Result<Self> {
        let bdaddr = parse_bdaddr(addr)?;
        // SAFETY: plain syscalls with properly initialised arguments.
        unsafe {
            let fd = libc::socket(
                libc::AF_BLUETOOTH,
                libc::SOCK_STREAM | libc::SOCK_CLOEXEC,
                BTPROTO_RFCOMM,
            );
            if fd < 0 {
                return Err(io::Error::last_os_error()).context("creating RFCOMM socket");
            }
            let stream = RfcommStream { fd };
            stream.set_timeouts(timeout)?;

            let sa = SockaddrRc {
                family: libc::AF_BLUETOOTH as libc::sa_family_t,
                bdaddr,
                channel,
            };
            let rc = libc::connect(
                fd,
                &sa as *const SockaddrRc as *const libc::sockaddr,
                std::mem::size_of::<SockaddrRc>() as libc::socklen_t,
            );
            if rc < 0 {
                return Err(io::Error::last_os_error()).with_context(|| {
                    format!("connecting to {addr} on RFCOMM channel {channel} (paired? in range? not connected to another host?)")
                });
            }
            Ok(stream)
        }
    }

    fn set_timeouts(&self, timeout: Duration) -> Result<()> {
        let tv = libc::timeval {
            tv_sec: timeout.as_secs() as libc::time_t,
            tv_usec: timeout.subsec_micros() as libc::suseconds_t,
        };
        for opt in [libc::SO_SNDTIMEO, libc::SO_RCVTIMEO] {
            // SAFETY: tv outlives the call, size matches.
            let rc = unsafe {
                libc::setsockopt(
                    self.fd,
                    libc::SOL_SOCKET,
                    opt,
                    &tv as *const libc::timeval as *const libc::c_void,
                    std::mem::size_of::<libc::timeval>() as libc::socklen_t,
                )
            };
            if rc < 0 {
                return Err(io::Error::last_os_error()).context("setting socket timeout");
            }
        }
        Ok(())
    }

    /// Change the receive timeout (used by `listen` / reply reading).
    pub fn set_read_timeout(&self, timeout: Duration) -> Result<()> {
        self.set_timeouts(timeout)
    }
}

impl Read for RfcommStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // SAFETY: buf is valid for buf.len() bytes.
        let n = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(n as usize)
        }
    }
}

impl Write for RfcommStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // SAFETY: buf is valid for buf.len() bytes.
        let n = unsafe { libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len()) };
        if n < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(n as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for RfcommStream {
    fn drop(&mut self) {
        // SAFETY: fd is owned by us.
        unsafe {
            libc::close(self.fd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bdaddr_is_reversed() {
        assert_eq!(
            parse_bdaddr("11:22:33:44:55:66").unwrap(),
            [0x66, 0x55, 0x44, 0x33, 0x22, 0x11]
        );
        assert!(parse_bdaddr("11:22:33").is_err());
        assert!(parse_bdaddr("zz:22:33:44:55:66").is_err());
    }
}
