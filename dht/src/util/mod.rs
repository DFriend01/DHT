use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::net::SocketAddr;
use std::path::Path;

pub fn read_socket_addresses(server_file_path: &Path) -> Result<Vec<SocketAddr>, io::Error> {
    let file: File = File::open(server_file_path)?;
    let reader: BufReader<File> = BufReader::new(file);
    let mut addresses: Vec<SocketAddr> = Vec::new();

    for line in reader.lines() {
        let line: String = line?;
        match line.parse::<SocketAddr>() {
            Ok(addr) => addresses.push(addr),
            Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid socket address")),
        }
    }

    Ok(addresses)
}
