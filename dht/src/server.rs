use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;
use std::time::Duration;
use crate::comm::UdpInterface;

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

    let timeout: Duration = Duration::from_millis(100);
    let max_retries: u32 = 3;
    let udp_interface: UdpInterface = match UdpInterface::new(server_addr, timeout, max_retries) {
        Ok(receiver) => receiver,
        Err(e) => {
            eprintln!("UdpInterface failed to bind to {}", SERVER_ADDR_STR);
            return Err(e);
        }
    };

    loop {
        let mut buffer: [u8; 1024] = [0; 1024];
        let (size, addr) = udp_interface.listen(&mut buffer, None)?;

        println!("Received {} bytes from {}", size, addr);
        let _ = udp_interface.send(&buffer, addr);
    }
}
