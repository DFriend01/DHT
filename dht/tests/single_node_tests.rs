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
fn test01_Ping_Success() {
    let result = common::ping_servers(vec![*SERVER_ADDR], false);
    assert!(result.is_ok());
}

#[test]
fn test02_Put_Get_Success() {
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

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn test03_Get_KeyNotFound() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let proto_interface = common::get_proto_interface().unwrap();
    let key: Vec<u8> = common::get_rand_key();

    let mut request: Request = Request::new();
    request.operation = Operation::Get as u32;
    request.key = Some(key.clone());

    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, *SERVER_ADDR).unwrap();
    let reply: Reply = extract_reply(&reply_msg).unwrap();
    assert_eq!(reply.status, Status::KeyNotFound as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn test04_Wipe_Success() {
    const NUM_KEY_VALUE_PAIRS: usize = 10;
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let proto_interface = common::get_proto_interface().unwrap();

    let mut key_value_pairs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for _ in 0..NUM_KEY_VALUE_PAIRS {
        key_value_pairs.push((common::get_rand_key(), common::get_rand_value()));
    }

    for (key, value) in key_value_pairs.iter() {
        let mut request: Request = Request::new();
        request.operation = Operation::Put as u32;
        request.key = Some(key.to_vec());
        request.value = Some(value.to_vec());

        let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, *SERVER_ADDR).unwrap();
        let reply: Reply = extract_reply(&reply_msg).unwrap();
        assert_eq!(reply.status, Status::Success as u32);
    }

    let result = common::wipe_servers(vec![*SERVER_ADDR], 1);
    assert!(result.is_ok());

    for (key, _) in key_value_pairs.iter() {
        let mut request: Request = Request::new();
        request.operation = Operation::Get as u32;
        request.key = Some(key.to_vec());

        let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, *SERVER_ADDR).unwrap();
        let reply: Reply = extract_reply(&reply_msg).unwrap();
        assert_eq!(reply.status, Status::KeyNotFound as u32);
    }
}
