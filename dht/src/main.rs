use clap::Parser;
use log;
use log::LevelFilter;
use std::net::SocketAddr;

use crate::logging::server::init_logger;
use crate::server::data::Node;

pub mod comm;
pub mod logging;
pub mod server;
pub mod util;

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Server ID
    #[arg(short, long, default_value = "0")]
    server_id: u32,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Max memory in megabytes
    #[arg(short, long, default_value = "32")]
    max_memory: u32,
}

fn main() {
    // Parse the command-line arguments
    let args = Args::parse();

    // Set the log level
    let log_level = match args.log_level.as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => {
            eprintln!("Invalid log level: {}", args.log_level);
            return;
        }
    };

    let server_addr_str = format!("127.0.0.1:{}", args.port);
    let server_addr: SocketAddr = match server_addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address {}", e);
            return;
        }
    };

    log::set_max_level(log_level);
    init_logger(log_level, args.server_id);

    let mut server: Node = match Node::new(server_addr, args.server_id, args.max_memory) {
        Ok(node) => node,
        Err(e) => {
            eprintln!("Failed to create server: {}", e);
            return;
        }
    };

    log::info!("Server N{} bound to address {}", args.server_id, server_addr);

    let _ = server.run();
}
