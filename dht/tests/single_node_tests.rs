#![allow(non_snake_case)]

use ctor::ctor;
use lazy_static::lazy_static;
use std::path::Path;
use std::net::SocketAddr;

use dht::util::read_socket_addresses;
use dht::comm::proto::Status;

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

    let (key, value, status) = common::put_rand_key_value(*SERVER_ADDR).unwrap();
    assert_eq!(status, Status::Success as u32);

    let (retrived_value_opt, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::Success as u32);

    let retrived_value: Vec<u8> = retrived_value_opt.ok_or("GET failed. No value present").unwrap();
    assert_eq!(retrived_value, value);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn test03_Get_KeyNotFound() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let key: Vec<u8> = common::get_rand_key();

    let (_, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::KeyNotFound as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn test04_Put_Get_Wipe_Success() {
    const NUM_KEY_VALUE_PAIRS: usize = 10;
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);

    let mut key_value_pairs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for _ in 0..NUM_KEY_VALUE_PAIRS {
        let (key, value, status) = common::put_rand_key_value(*SERVER_ADDR).unwrap();
        assert_eq!(status, Status::Success as u32);

        let (retrived_value_opt, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
        assert_eq!(status, Status::Success as u32);

        let retrived_value: Vec<u8> = retrived_value_opt.ok_or("GET failed. No value present").unwrap();
        assert_eq!(retrived_value, value);

        key_value_pairs.push((key, value));
    }

    let result = common::wipe_servers(vec![*SERVER_ADDR], 1);
    assert!(result.is_ok());

    for (key, _) in key_value_pairs.iter() {
        let (_, status) = common::get_value(*SERVER_ADDR, key).unwrap();
        assert_eq!(status, Status::KeyNotFound as u32);
    }
}
