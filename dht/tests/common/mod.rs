#![allow(dead_code)]

use lazy_static::lazy_static;
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

lazy_static! {
    static ref CLIENT_ADDR: SocketAddr = {
        UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap()
    };
}

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
    Ok(ProtoInterface::new(*CLIENT_ADDR)?)
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

pub fn shutdown_servers(server_addrs: Vec<SocketAddr>, wait_time_sec: u64) -> Result<()> {
    let proto_interface = get_proto_interface()?;
    let mut failed = false;

    for server_addr in server_addrs {
        log::info!("Shutting down server at {}", server_addr);
        let mut request: Request = Request::new();
        request.operation = Operation::Shutdown as u32;
        let (reply_msg, _) = proto_interface.send_and_recv(request, server_addr)?;
        let reply: Reply = extract_reply(&reply_msg)?;
        if reply.status != Status::Success as u32 {
            log::error!("Shutdown failed for server at {} with status code {:?}", server_addr, reply.status);
            failed = true;
        }
    }

    log::info!("Waiting for {} seconds...", wait_time_sec);
    std::thread::sleep(std::time::Duration::from_secs(wait_time_sec));

    if failed {
        Err(Error::new(ErrorKind::Other, "Shutdown failed"))
    } else {
        Ok(())
    }
}

pub fn put_key_value(server_addr: SocketAddr, key: &Option<Vec<u8>>, value: &Option<Vec<u8>>) -> Result<u32> {

    log::debug!("Getting the proto interface...");
    let proto_interface = get_proto_interface()?;

    let mut request: Request = Request::new();
    request.operation = Operation::Put as u32;
    request.key = key.clone();
    request.value = value.clone();

    log::debug!("Starting send and receive of PUT key request...");
    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, server_addr)?;
    let reply: Reply = extract_reply(&reply_msg)?;

    Ok(reply.status)
}

pub fn put_rand_key_value(server_addr: SocketAddr) -> Result<(Vec<u8>, Vec<u8>, u32)> {
    let key: Vec<u8> = get_rand_key();
    let value: Vec<u8> = get_rand_value();
    let status = put_key_value(server_addr, &Some(key.clone()), &Some(value.clone()))?;
    Ok((key, value, status))
}

pub fn get_value(server_addr: SocketAddr, key: &Vec<u8>) -> Result<(Option<Vec<u8>>, u32)> {
    let proto_interface = get_proto_interface()?;

    let mut request: Request = Request::new();
    request.operation = Operation::Get as u32;
    request.key = Some(key.to_vec());

    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, server_addr)?;
    let reply: Reply = extract_reply(&reply_msg)?;

    Ok((reply.value, reply.status))
}

pub fn delete_key_value(server_addr: SocketAddr, key: &Vec<u8>) -> Result<(Option<Vec<u8>>, u32)> {
    let proto_interface = get_proto_interface()?;

    let mut request: Request = Request::new();
    request.operation = Operation::Delete as u32;
    request.key = Some(key.to_vec());

    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, server_addr)?;
    let reply: Reply = extract_reply(&reply_msg)?;

    Ok((reply.value, reply.status))
}

pub fn get_rand_bytes(min_len: usize, max_len: usize) -> Vec<u8> {
    let len = rand::thread_rng().gen_range(min_len..max_len);
    get_bytes(len)
}

pub fn get_bytes(len: usize) -> Vec<u8> {
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
