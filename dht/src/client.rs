use std::net::SocketAddr;
use std::io::{Error, ErrorKind};

use comm::proto::Operation;

use crate::comm::ProtoInterface;
use crate::comm::protogen::api::Request;

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

    let mut request: Request = Request::new();
    request.operation = Operation::Ping as u32;

    let (_reply, addr) = client.send_and_recv(request, server_addr)?;
    println!("Received bytes from {}", addr);

    Ok(())
}
