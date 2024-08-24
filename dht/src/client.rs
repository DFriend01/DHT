use std::net::SocketAddr;
use std::io::{Error, ErrorKind};

use crate::comm::proto;
use crate::comm::ProtoInterface;
use crate::comm::protogen::api::UDPMessage;

pub mod comm;

fn main() -> std::io::Result<()> {

    let client_addr: SocketAddr = match "0.0.0.0:0".parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address: {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    let client: ProtoInterface = ProtoInterface::new(client_addr)?;

    println!("Bound to address: {}", client_addr);

    let server_addr: SocketAddr = match "127.0.0.1:8080".parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address: {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    let id: Vec<u8> = b"wqerwer".to_vec();
    let payload: Vec<u8> = b"Hello, world!".to_vec();
    let checksum: u64 = proto::calculate_checksum(id.as_slice(), payload.as_slice());
    let mut message: UDPMessage = UDPMessage::new();
    message.id = id;
    message.payload = payload;
    message.checksum = checksum;

    let (size, addr) = client.send_and_recv(message, server_addr)?;
    println!("Received {} bytes from {}", size, addr);

    Ok(())
}
