use std::io::{Result, Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

pub mod protos;

const TIMEOUT_MULTIPLIER: u32 = 2;

pub struct UdpClient {
    socket: UdpSocket,
    timeout: Duration,
    max_retries: u32,
}

impl UdpClient {
    pub fn new(socket_addr: SocketAddr, timeout: Duration, max_retries: u32) -> Result<Self> {
        let socket: UdpSocket = match UdpSocket::bind(socket_addr) {
            Ok(socket) => socket,
            Err(e) => return Err(e),
        };
        Ok(UdpClient {socket, timeout, max_retries})
    }

    pub fn send(&self, message: &[u8], server_addr: SocketAddr) -> Result<usize> {
        self.socket.send_to(message, server_addr)
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.socket.set_read_timeout(Some(self.timeout))?;
        self.socket.recv_from(buf)
    }

    pub fn send_and_recv(&self, message: &[u8], server_addr: SocketAddr, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.socket.set_read_timeout(Some(self.timeout))?;
        self.do_send_and_recv(message, server_addr, buf)
    }

    fn do_send_and_recv(&self, message: &[u8], server_addr: SocketAddr, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let max_attempted_sends: u32 = self.max_retries + 1;
        let timeout: Duration = self.timeout;

        for _ in 0..max_attempted_sends {
            match self.socket.send_to(message, server_addr) {
                Ok(_size) => {
                    match self.socket.recv_from(buf) {
                        Ok((size, addr)) => return Ok((size, addr)),
                        Err(e) => {
                            let timeout = timeout.checked_mul(TIMEOUT_MULTIPLIER).unwrap();
                            if e.kind() == ErrorKind::TimedOut {
                                self.socket.set_read_timeout(Some(timeout))?;
                                continue;
                            }
                            return Err(e);
                        }
                    }
                },
                Err(e) => return Err(e),
            }
        }

        Err(Error::new(ErrorKind::TimedOut, "Timed out"))
    }
}
