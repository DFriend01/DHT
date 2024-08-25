use std::net::SocketAddr;
use log;

use crate::logging::server::init_logger;
use crate::server::data::Node;

pub mod comm;
pub mod logging;
pub mod server;

fn main() {
    const SERVER_ADDR_STR: &str = "127.0.0.1:8080";
    let server_addr: SocketAddr = match SERVER_ADDR_STR.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address {}", e);
            return;
        }
    };

    let id: u32 = 0;
    init_logger(log::LevelFilter::Debug, id);

    let mut server: Node = match Node::new(server_addr, id) {
        Ok(node) => node,
        Err(e) => {
            eprintln!("Failed to create server: {}", e);
            return;
        }
    };

    log::info!("Server N{} bound to address {}", id, server_addr);

    let _ = server.run();
}
