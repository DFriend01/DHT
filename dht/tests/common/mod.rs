use log;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
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

pub fn ping_servers(server_addrs: Vec<SocketAddr>, should_panic_if_fail: bool) -> Result<()> {
    let client_addr: SocketAddr = UdpSocket::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();

    let proto_interface = match ProtoInterface::new(client_addr) {
        Ok(proto_interface) => proto_interface,
        Err(e) => {
            if should_panic_if_fail {
                panic!("Failed to create ProtoInterface: {:?}", e);
            } else {
                return Err(e);
            }
        }
    };

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
