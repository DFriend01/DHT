#![allow(non_snake_case)]

use ctor::ctor;
use lazy_static::lazy_static;
use std::path::Path;
use std::net::SocketAddr;

use dht::util::read_socket_addresses;
use dht::comm::proto::{Operation, Status, extract_reply};
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
fn test01_ping() {
    let result = common::ping_servers(vec![*SERVER_ADDR], false);
    assert!(result.is_ok());
}

#[test]
fn test02_Op_PUT_GET() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let proto_interface = common::get_proto_interface().unwrap();
    let key: Vec<u8> = common::get_rand_key();
    let value: Vec<u8> = common::get_rand_value();

    let mut request: Request = Request::new();
    request.operation = Operation::Put as u32;
    request.key = Some(key.clone());
    request.value = Some(value.clone());

    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, *SERVER_ADDR).unwrap();
    let reply: Reply = extract_reply(&reply_msg).unwrap();
    assert_eq!(reply.status, Status::Success as u32);

    let mut request: Request = Request::new();
    request.operation = Operation::Get as u32;
    request.key = Some(key.clone());

    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, *SERVER_ADDR).unwrap();
    let reply: Reply = extract_reply(&reply_msg).unwrap();
    assert_eq!(reply.status, Status::Success as u32);

    let retrived_value: Vec<u8> = reply.value.ok_or("GET failed. No value present").unwrap();
    assert_eq!(retrived_value, value);
}
