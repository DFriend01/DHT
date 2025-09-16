use core::option::Option;
use std::io::{Result, Error, ErrorKind};
use std::net::{SocketAddr, IpAddr, UdpSocket};
use std::time::Duration;
use protobuf::Message;

use crate::comm::proto::parse_message;
use crate::comm::protogen::api::{UDPMessage};

pub mod proto;
pub mod protogen;

const SEND_RECV_TIMEOUT: Duration = Duration::from_millis(100);
const LISTENING_TIMEOUT: Duration = Duration::from_millis(1000);
const MAX_RETRIES: u32 = 3;
const MAX_BUFFER_SIZE_BYTES: usize = 1024 * 12;
const TIMEOUT_MULTIPLIER: u32 = 2;

pub struct ProtoInterface {
    udp_interface: UdpInterface,
    ip: IpAddr,
    port: u16,
}

impl ProtoInterface {
    pub fn new(socket_addr: SocketAddr) -> Result<Self> {
        let udp_interface: UdpInterface = UdpInterface::new(
            socket_addr,
            Some(SEND_RECV_TIMEOUT),
            Some(LISTENING_TIMEOUT),
            MAX_RETRIES
        )?;
        let ip = socket_addr.ip();
        let port = socket_addr.port();
        Ok(ProtoInterface {udp_interface, ip, port})
    }

    pub fn send(&self, message: impl Message, server_addr: SocketAddr) -> Result<usize> {
        let udp_message: UDPMessage = proto::create_udp_message(message, self.ip, self.port)?;
        let msg_bytes: Vec<u8> = UDPMessage::write_to_bytes(&udp_message)?;
        self.udp_interface.send(msg_bytes.as_slice(), server_addr)
    }

    pub fn listen(&self) -> Result<(UDPMessage, SocketAddr)> {
        let mut buf: [u8; MAX_BUFFER_SIZE_BYTES] = [0; MAX_BUFFER_SIZE_BYTES];
        let (size, sender_addr) = match self.udp_interface.listen(&mut buf) {
            Ok((size, sender_addr)) => (size, sender_addr),
            Err(e) => {
                return Err(e);
            }
        };

        let recv_data: Vec<u8> = buf[0..size].to_vec();
        let message: UDPMessage = parse_message(recv_data)?;

        match proto::validate_checksum(&message) {
            Ok(_) => Ok((message, sender_addr)),
            Err(e) => Err(e)
        }
    }

    pub fn send_and_recv(&self, message: impl Message, server_addr: SocketAddr) -> Result<(UDPMessage, SocketAddr)> {
        let udp_message: UDPMessage = proto::create_udp_message(message, self.ip, self.port)?;
        let msg_bytes: Vec<u8> = UDPMessage::write_to_bytes(&udp_message)?;
        let mut buf: [u8; MAX_BUFFER_SIZE_BYTES] = [0; MAX_BUFFER_SIZE_BYTES];
        let (size, sender_addr) = self.udp_interface.send_and_recv(&msg_bytes, server_addr, &mut buf)?;

        let recv_data: Vec<u8> = buf[0..size].to_vec();
        let message: UDPMessage = parse_message(recv_data)?;

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

#[cfg(test)]
mod tests {
    use protogen::api::Request;

    use super::*;
    use crate::comm::proto::Operation;

    fn create_client_and_server() -> (ProtoInterface, SocketAddr, ProtoInterface, SocketAddr) {
        let server_addr: SocketAddr = UdpSocket::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        let client_addr: SocketAddr = UdpSocket::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        let server_interface: ProtoInterface = ProtoInterface::new(server_addr).unwrap();
        let client_interface: ProtoInterface = ProtoInterface::new(client_addr).unwrap();

        (client_interface, client_addr, server_interface, server_addr)
    }

    #[test]
    fn test_proto_interface_receive() {
        let (client_interface, client_addr, server_interface, server_addr) = create_client_and_server();

        let mut sent_request: Request = Request::new();
        sent_request.operation = Operation::Ping as u32;
        client_interface.send(sent_request.clone(), server_addr).unwrap();

        let (received_message, sender_socket) = server_interface.listen().unwrap();
        let received_request = Request::parse_from_bytes(&received_message.payload).unwrap();

        assert_eq!(received_request, sent_request);
        assert_eq!(sender_socket, client_addr);
    }

    #[test]
    fn test_proto_interface_failed_receive() {
        let (client_interface, _, _, server_addr) = create_client_and_server();
        let mut sent_request: Request = Request::new();
        sent_request.operation = Operation::Ping as u32;
        let result = client_interface.send_and_recv(sent_request.clone(), server_addr);
        assert!(result.is_err());
    }
}
