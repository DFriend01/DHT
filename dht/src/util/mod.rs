use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::net::SocketAddr;
use std::path::Path;

pub fn read_socket_addresses(server_file_path: &Path) -> Result<Vec<SocketAddr>, io::Error> {
    let file: File = File::open(server_file_path)?;
    let reader: BufReader<File> = BufReader::new(file);
    let mut addresses: Vec<SocketAddr> = Vec::new();

    for line in reader.lines() {
        let line: String = line?.trim().to_string();
        match line.parse::<SocketAddr>() {
            Ok(addr) => addresses.push(addr),
            Err(_) => {
                if !line.is_empty() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid socket address"))
                }
            },
        }
    }

    Ok(addresses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;
    use std::io::Write;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_read_socket_addresses() {
       let test_file_path = "test_read_socket_addresses.txt";
        let mut file = File::create(test_file_path).unwrap();
        let addrs: [SocketAddr; 2] = [
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
        ];
        writeln!(file, "{}", format!("{}\n{}", addrs[0], addrs[1])).unwrap();

        let result = read_socket_addresses(Path::new(test_file_path)).unwrap();
        remove_file(test_file_path).unwrap();

        assert_eq!(result, addrs);
    }

    #[test]
    fn test_read_socket_addresses_with_newlines() {
       let test_file_path = "test_read_socket_addresses.txt";
        let mut file = File::create(test_file_path).unwrap();
        let addrs: [SocketAddr; 2] = [
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
        ];
        writeln!(file, "{}", format!("{}\n{}\n\n", addrs[0], addrs[1])).unwrap();

        let result = read_socket_addresses(Path::new(test_file_path)).unwrap();
        remove_file(test_file_path).unwrap();

        assert_eq!(result, addrs);
    }

    #[test]
    fn test_read_socket_addresses_with_spaces() {
       let test_file_path = "test_read_socket_addresses.txt";
        let mut file = File::create(test_file_path).unwrap();
        let addrs: [SocketAddr; 2] = [
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
        ];
        writeln!(file, "{}", format!(" {} \n{} \n ", addrs[0], addrs[1])).unwrap();

        let result = read_socket_addresses(Path::new(test_file_path)).unwrap();
        remove_file(test_file_path).unwrap();

        assert_eq!(result, addrs);
    }
}
