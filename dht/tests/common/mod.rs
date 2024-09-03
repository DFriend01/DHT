use log::{self, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use rand::{self, Rng, RngCore};
use std::io::{Error, ErrorKind, Result};
use std::net::{SocketAddr, UdpSocket};

use dht::comm::ProtoInterface;
use dht::comm::proto::{extract_reply, Operation, Status};
use dht::comm::protogen::api::{Request, Reply};

pub fn init_logger() {
    // Pattern
    let pattern: &str = "[{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {l}] {m}{n}";

    // Appenders
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&pattern)))
        .build();

    // Initialize the loggers
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}

pub fn get_proto_interface() -> Result<ProtoInterface> {
    let client_addr: SocketAddr = UdpSocket::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
    Ok(ProtoInterface::new(client_addr)?)
}

pub fn ping_servers(server_addrs: Vec<SocketAddr>, should_panic_if_fail: bool) -> Result<()> {
    let proto_interface = get_proto_interface()?;

    for server_addr in server_addrs {
        log::info!("Pinging server at {}", server_addr);
        let mut request: Request = Request::new();
        request.operation = Operation::Ping as u32;
        let (reply_msg, _) = match proto_interface.send_and_recv(request, server_addr) {
            Ok((reply, _server_socket)) => (reply, _server_socket),
            Err(e) => {
                if should_panic_if_fail {
                    panic!("Ping failed: {:?}", e);
                } else {
                    return Err(e);
                }
            }
        };

        let reply: Reply = match extract_reply(&reply_msg) {
            Ok(reply) => reply,
            Err(e) => {
                if should_panic_if_fail {
                    panic!("Ping failed: {:?}", e);
                } else {
                    return Err(e);
                }
            }
        };

        if reply.status != Status::Success as u32 {
            if should_panic_if_fail {
                panic!("Ping failed: {:?}", reply.status);
            } else {
                return Err(Error::new(ErrorKind::Other, "Ping did not return SUCCESS"));
            }
        }
    }
    Ok(())
}

pub fn wipe_servers(server_addrs: Vec<SocketAddr>, wait_time_sec: u64) -> Result<()> {
    let proto_interface = get_proto_interface()?;
    let mut failed = false;

    for server_addr in server_addrs {
        log::info!("Wiping server at {}", server_addr);
        let mut request: Request = Request::new();
        request.operation = Operation::Wipe as u32;
        let (reply_msg, _) = proto_interface.send_and_recv(request, server_addr)?;
        let reply: Reply = extract_reply(&reply_msg)?;
        if reply.status != Status::Success as u32 {
            log::error!("Wipe failed for server at {} with status code {:?}", server_addr, reply.status);
            failed = true;
        }
    }

    log::info!("Waiting for {} seconds...", wait_time_sec);
    std::thread::sleep(std::time::Duration::from_secs(wait_time_sec));

    if failed {
        Err(Error::new(ErrorKind::Other, "Wipe failed"))
    } else {
        Ok(())
    }
}

pub fn get_rand_bytes(min_len: usize, max_len: usize) -> Vec<u8> {
    let len = rand::thread_rng().gen_range(min_len..max_len);
    let mut bytes = vec![0; len];
    let _ = rand::thread_rng().try_fill_bytes(&mut bytes);
    bytes
}

pub fn get_rand_key() -> Vec<u8> {
    get_rand_bytes(8, 32)
}

pub fn get_rand_value() -> Vec<u8> {
    get_rand_bytes(8, 1024)
}
