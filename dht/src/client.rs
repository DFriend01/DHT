use core::option::Option;
use std::net::SocketAddr;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use crate::comm::UdpInterface;

pub mod comm;

fn main() -> std::io::Result<()> {

    let client_addr: SocketAddr = match "0.0.0.0:0".parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address: {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    println!("Bound to address: {}", client_addr);

    let timeout: Option<Duration> = Some(Duration::from_millis(100));
    let listener_timeout: Option<Duration> = None;
    let max_retries: u32 = 3;
    let client: UdpInterface = UdpInterface::new(
        client_addr,
        timeout,
        listener_timeout,
        max_retries
    )?;

    let server_addr: SocketAddr = match "127.0.0.1:8080".parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address: {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    let message: &[u8] = b"Hello, world!";
    let mut buffer: [u8; 1024] = [0; 1024];

    let (size, addr) = client.send_and_recv(message, server_addr, &mut buffer)?;
    println!("Received {} bytes from {}", size, addr);

    Ok(())
}
