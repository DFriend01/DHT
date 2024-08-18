use std::net::{SocketAddr, UdpSocket};
use std::io::{Error, ErrorKind};

fn main() -> std::io::Result<()> {
    // Create a UDP socket
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0")?;

    // Server address to ping
    let server_addr: SocketAddr = match "127.0.0.1:8080".parse::<SocketAddr>() {
        Ok(addr) => addr,
        Err(e) =>  {
            return Err(Error::new(ErrorKind::InvalidInput, e))
        }
    };

    // Message to send
    let message: &str = "Ping";

    // Send the message to the server
    socket.send_to(message.as_bytes(), server_addr)?;

    // buf to store the response
    let mut buf: [u8; 1024] = [0; 1024];

    // Receive the response from the server
    let (size, _) = socket.recv_from(&mut buf)?;

    // Print the response
    let response: std::borrow::Cow<str> = String::from_utf8_lossy(&buf[..size]);
    println!("Response from server: {}", response);

    Ok(())
}
