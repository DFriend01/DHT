use std::net::SocketAddr;
use log::{LevelFilter, info, warn};

use crate::logging::server::init_logger;
use crate::server::data::Node;

pub mod comm;
pub mod logging;
pub mod server;

fn main() {
    const SERVER_ADDR_STR: &str = "127.0.0.1:8080";
    let server_addr: SocketAddr = match SERVER_ADDR_STR.parse() {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("Failed to parse address {}", SERVER_ADDR_STR);
            return;
        }
    };

    let mut server: Node = match Node::new(server_addr) {
        Ok(node) => node,
        Err(e) => {
            eprintln!("Failed to create server: {}", e);
            return;
        }
    };

    init_logger(LevelFilter::Debug, 0);
    info!("Server started at {}", server_addr);
    warn!("This is a warning message");

    let _ = server.run();
}
