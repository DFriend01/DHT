use core::option::Option;
use std::io::{Result, Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use protobuf::Message;

use crate::comm::protogen::api::UDPMessage;

pub mod proto;
pub mod protogen;

const SEND_RECV_TIMEOUT: Duration = Duration::from_millis(100);
const LISTENING_TIMEOUT: Duration = Duration::from_millis(1000);
const MAX_RETRIES: u32 = 3;
const TIMEOUT_MULTIPLIER: u32 = 2;

pub struct ProtoInterface {
    udp_interface: UdpInterface
}

impl ProtoInterface {
    pub fn new(socket_addr: SocketAddr) -> Result<Self> {

        let udp_interface: UdpInterface = UdpInterface::new(
            socket_addr,
            Some(SEND_RECV_TIMEOUT),
            Some(LISTENING_TIMEOUT),
            MAX_RETRIES
        )?;
        Ok(ProtoInterface {udp_interface})
    }

    pub fn send(&self, message: UDPMessage, server_addr: SocketAddr) -> Result<usize> {
        let msg_bytes: Vec<u8> = UDPMessage::write_to_bytes(&message)?;
        self.udp_interface.send(msg_bytes.as_slice(), server_addr)
    }

    pub fn listen(&self) -> Result<(UDPMessage, SocketAddr)> {
        let mut buf: [u8; 1024] = [0; 1024];
        let (size, sender_addr) = self.udp_interface.listen(&mut buf)?;

        let recv_data: Vec<u8> = buf[0..size].to_vec();
        let message: UDPMessage = UDPMessage::parse_from_bytes(recv_data.as_slice())?;

        match proto::validate_checksum(&message) {
            Ok(_) => Ok((message, sender_addr)),
            Err(e) => Err(e),
        }
    }

    pub fn send_and_recv(&self, message: UDPMessage, server_addr: SocketAddr) -> Result<(UDPMessage, SocketAddr)> {
        let msg_bytes: Vec<u8> = UDPMessage::write_to_bytes(&message)?;
        let mut buf: [u8; 1024] = [0; 1024];
        let (size, sender_addr) = self.udp_interface.send_and_recv(&msg_bytes, server_addr, &mut buf)?;

        let recv_data: Vec<u8> = buf[0..size].to_vec();
        let message: UDPMessage = UDPMessage::parse_from_bytes(recv_data.as_slice())?;

        match proto::validate_checksum(&message) {
            Ok(_) => Ok((message, sender_addr)),
            Err(e) => Err(e),
        }
    }
}

struct UdpInterface {
    socket: UdpSocket,
    send_recv_timeout: Option<Duration>,
    listening_timeout: Option<Duration>,
    max_retries: u32,
}

impl UdpInterface {
    pub fn new(
        socket_addr: SocketAddr,
        send_recv_timeout: Option<Duration>,
        listening_timeout: Option<Duration>,
        max_retries: u32
    ) -> Result<Self> {
        let socket: UdpSocket = match UdpSocket::bind(socket_addr) {
            Ok(socket) => socket,
            Err(e) => return Err(e),
        };
        Ok(UdpInterface {socket, send_recv_timeout, listening_timeout, max_retries})
    }

    pub fn send(&self, message: &[u8], server_addr: SocketAddr) -> Result<usize> {
        self.socket.send_to(message, server_addr)
    }

    pub fn listen(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.socket.set_read_timeout(self.listening_timeout)?;
        self.socket.recv_from(buf)
    }

    pub fn send_and_recv(&self, message: &[u8], server_addr: SocketAddr, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.socket.set_read_timeout(self.send_recv_timeout)?;
        self.do_send_and_recv(message, server_addr, buf)
    }

    fn do_send_and_recv(&self, message: &[u8], server_addr: SocketAddr, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let max_attempted_sends: u32 = self.max_retries + 1;
        let timeout: Duration = match self.send_recv_timeout {
            Some(t) => t,
            None => Duration::from_millis(100),
        };

        for _ in 0..max_attempted_sends {
            match self.socket.send_to(message, server_addr) {
                Ok(_size) => {
                    match self.socket.recv_from(buf) {
                        Ok((size, addr)) => {
                            return Ok((size, addr))
                        },
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
                Err(e) => {
                    return Err(e)
                },
            }
        }

        Err(Error::new(ErrorKind::TimedOut, "Timed out"))
    }
}
