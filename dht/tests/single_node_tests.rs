use std::path::Path;
use std::net::SocketAddr;

use dht::util::read_socket_addresses;

mod common;

#[test]
fn ping() {
    let server_file_path: &Path = Path::new("servers.txt");
    let server_addr: SocketAddr = read_socket_addresses(server_file_path).unwrap()[0];
    let result = common::ping_servers(vec![server_addr], false);
    assert!(result.is_ok());
}
