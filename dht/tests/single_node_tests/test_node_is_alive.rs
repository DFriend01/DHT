#![allow(non_snake_case)]

use ctor::ctor;
use lazy_static::lazy_static;
use std::path::Path;
use std::net::SocketAddr;

use dht::util::read_socket_addresses;
use dht::comm::proto::{extract_reply, Operation, Status};
use dht::comm::protogen::api::{Request, Reply};

mod common;

const SERVER_FILE: &str = "servers/single_server.txt";
lazy_static! {
    static ref SERVER_ADDR: SocketAddr = read_socket_addresses(Path::new(SERVER_FILE)).unwrap()[0];
}

#[ctor]
fn init() {
    common::init_logger();
}

#[test]
fn Ping_Success() {
    let result = common::ping_servers(vec![*SERVER_ADDR], false);
    assert!(result.is_ok());
}
