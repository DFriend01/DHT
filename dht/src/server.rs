use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;
use comm::ProtoInterface;

// use crate::comm::protogen::api::{UDPMessage, Request, Reply};

pub mod comm;

fn main() -> Result<()> {
    const SERVER_ADDR_STR: &str = "127.0.0.1:8080";
    let server_addr: SocketAddr = match SERVER_ADDR_STR.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address {}", SERVER_ADDR_STR);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    let server: ProtoInterface = match ProtoInterface::new(server_addr) {
        Ok(receiver) => receiver,
        Err(e) => {
            return Err(e);
        }
    };

    loop {
        let (msg, addr) = match server.listen() {
            Ok((msg, addr)) => (msg, addr),
            Err(e) => {
                eprintln!("Failed to receive message: {}", e);
                continue;
            }
        };

        println!("Received bytes from {}", addr);
        let _ = server.send(msg, addr);
    }
}
