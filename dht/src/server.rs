use std::net::UdpSocket;

fn main() {
    const SERVER_ADDR_STR: &str = "127.0.0.1:8080";
    let socket: UdpSocket = match UdpSocket::bind(SERVER_ADDR_STR) {
        Ok(socket) => {
            println!("Bound to address: {}", SERVER_ADDR_STR);
            socket
        },
        Err(e) => {
            eprintln!("Failed to bind to address: {}", e);
            return;
        }
    };

    loop {
        let mut buffer: [u8; 1024] = [0; 1024];
        let (size, addr) = match socket.recv_from(&mut buffer) {
            Ok((size, addr)) =>  {
                (size, addr)
            },
            Err(e) => {
                eprintln!("Failed to receive data: {}", e);
                continue;
            }
        };

        println!("Received {} bytes from {}", size, addr);
        let _ = socket.send_to(&buffer, addr);
    }
}
