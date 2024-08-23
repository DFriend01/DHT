use std::net::SocketAddr;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use crate::comm::UdpSender;

pub mod comm;

fn main() -> std::io::Result<()> {

    let client_addr: SocketAddr = match "0.0.0.0:0".parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address: {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    let timeout: Duration = Duration::from_millis(100);
    let max_retries: u32 = 3;
    let client: UdpSender = UdpSender::new(client_addr, timeout, max_retries)?;

    let server_addr: SocketAddr = match "127.0.0.1:8080".parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address: {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    let message: &[u8] = b"Hello, world!";
    let mut buffer: [u8; 1024] = [0; 1024];

    let (size, addr) = client.send(message, server_addr, &mut buffer)?;
    println!("Received {} bytes from {}", size, addr);

    Ok(())
}
