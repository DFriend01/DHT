use clap::Parser;
use log;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::path::{PathBuf, Path};
use std::net::SocketAddr;

use crate::logging::server::init_logger;
use crate::server::data::Node;

pub mod comm;
pub mod logging;
pub mod server;
pub mod util;

// Command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    // Server ID
    #[arg(short, long, default_value = "0")]
    server_id: u32
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Args {{ port: {}, server_id: {} }}", self.port, self.server_id)
    }
}

// Configuration file fields
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    log_level: String,
    max_memory_mb: u32,
    chord_sizing_factor: usize
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let base_dir: &'static str = env!("CARGO_MANIFEST_DIR");
        let config_path: PathBuf = Path::new(base_dir).join("config.toml");
        let content: String = match std::fs::read_to_string(config_path) {
            Ok(deserialized_content) => deserialized_content,
            Err(e) => return Err(Box::new(e))
        };
        let config: Config = match toml::from_str(content.as_str()) {
            Ok(config) => config,
            Err(e) => return Err(Box::new(e))
        };
        Ok(config)
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Config {{ log_level: {}, max_memory_mb: {} }}", self.log_level, self.max_memory_mb)
    }
}

fn main() {
    let cli_args: Args = Args::parse();
    let config: Config = Config::load().expect("Unable to read and deserialize config.toml");

    // Set the log level
    let log_level = match config.log_level.as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => {
            eprintln!("Invalid log level: {}", config.log_level);
            return;
        }
    };

    let server_addr_str = format!("127.0.0.1:{}", cli_args.port);
    let server_addr: SocketAddr = match server_addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse address {}", e);
            return;
        }
    };

    log::set_max_level(log_level);
    init_logger(log_level, cli_args.server_id);

    let mut server: Node = match Node::new(server_addr, cli_args.server_id, config.max_memory_mb, config.chord_sizing_factor) {
        Ok(node) => node,
        Err(e) => {
            eprintln!("Failed to create server: {}", e);
            return;
        }
    };

    log::info!("{}", cli_args);
    log::info!("{}", config);
    log::info!("Server N{} bound to address {}", cli_args.server_id, server_addr);

    let _ = server.run();
}
