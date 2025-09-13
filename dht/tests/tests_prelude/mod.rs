pub use ctor::ctor;
pub use std::net::SocketAddr;

use lazy_static::lazy_static;
use std::path::Path;
use dht::util::read_socket_addresses;

const SERVER_FILE: &str = "servers/single_server.txt";
lazy_static! {
    pub static ref SERVER_ADDR: SocketAddr = read_socket_addresses(Path::new(SERVER_FILE)).unwrap()[0];
}
