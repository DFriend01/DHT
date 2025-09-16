#![allow(non_snake_case)]

use dht::comm::proto::{extract_reply, Operation, Status};
use dht::comm::protogen::api::{Request, Reply};

mod common;
mod tests_prelude;

use ntest::test_case;
use tests_prelude::*;

const KEY_VALUE_SIZE_BYTES: usize = 64;

#[ctor]
fn init() {
    common::init_logger();
}

// TODO: Add test to PUT increasingly larger packets up to a max

#[test]
fn GetPid_Success() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let proto_interface = common::get_proto_interface().unwrap();

    let mut request = Request::new();
    request.operation = Operation::GetPid as u32;

    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, *SERVER_ADDR).unwrap();
    let reply: Reply = extract_reply(&reply_msg).unwrap();

    let retrieved_pid: u32 = reply.pid.unwrap();
    log::info!("Received PID {}", retrieved_pid);

    assert_eq!(reply.status, Status::Success as u32);
}

#[test_case(64)]
#[test_case(128)]
#[test_case(256)]
#[test_case(512)]
#[test_case(1024)]
#[test_case(2048)]
#[test_case(4096)]
#[test_case(8192)]
fn Put_Get_Success(value_size: usize) {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);

    let key: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);
    let value: Vec<u8> = common::get_bytes(value_size);

    log::info!("Inserting key-value pair with Key Size {} B and Value Size {} B", KEY_VALUE_SIZE_BYTES, value_size);
    let status: u32 = common::put_key_value(*SERVER_ADDR, &Some(key.clone()), &Some(value.clone())).unwrap();
    assert_eq!(status, Status::Success as u32);

    let (retrived_value_opt, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::Success as u32);

    let retrived_value: Vec<u8> = retrived_value_opt.ok_or("GET failed. No value present").unwrap();
    assert_eq!(retrived_value, value);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Put_InvalidValueSize() {
    // Assuming the max value payload is 10KB defined in the node module
    const VALUE_SIZE_BYTES: usize = 11 * 1024;

    let _result = common::ping_servers(vec![*SERVER_ADDR], true);

    let key: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);
    let value: Vec<u8> = common::get_bytes(VALUE_SIZE_BYTES);

    let status: u32 = common::put_key_value(*SERVER_ADDR, &Some(key.clone()), &Some(value.clone())).unwrap();
    assert_eq!(status, Status::InvalidValueSize as u32);
}

#[test]
fn Put_MissingKey() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let key: Option<Vec<u8>> = None;
    let value: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);

    let status: u32 = common::put_key_value(*SERVER_ADDR, &key, &Some(value.clone())).unwrap();
    assert_eq!(status, Status::MissingKey as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Put_MissingValue() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let key: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);
    let value: Option<Vec<u8>> = None;

    let status: u32 = common::put_key_value(*SERVER_ADDR, &Some(key.clone()), &value).unwrap();
    assert_eq!(status, Status::MissingValue as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Get_MissingKey() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let proto_interface = common::get_proto_interface().unwrap();
    let key: Option<Vec<u8>> = None;

    let mut request = Request::new();
    request.operation = Operation::Get as u32;
    request.key = key;

    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, *SERVER_ADDR).unwrap();
    let reply: Reply = extract_reply(&reply_msg).unwrap();
    assert_eq!(reply.status, Status::MissingKey as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Get_KeyNotFound() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let key: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);

    let (_, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::KeyNotFound as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Delete_MissingKey() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let proto_interface = common::get_proto_interface().unwrap();
    let key: Option<Vec<u8>> = None;

    let mut request = Request::new();
    request.operation = Operation::Delete as u32;
    request.key = key;

    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, *SERVER_ADDR).unwrap();
    let reply: Reply = extract_reply(&reply_msg).unwrap();
    assert_eq!(reply.status, Status::MissingKey as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Put_Get_Wipe_Get_KeyNotFound() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);

    let key: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);
    let value: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);

    let status: u32 = common::put_key_value(*SERVER_ADDR, &Some(key.clone()), &Some(value.clone())).unwrap();
    assert_eq!(status, Status::Success as u32);

    let (retrived_value_opt, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::Success as u32);

    let retrived_value: Vec<u8> = retrived_value_opt.ok_or("GET failed. No value present").unwrap();
    assert_eq!(retrived_value, value);

    let result = common::wipe_servers(vec![*SERVER_ADDR], 1);
    assert!(result.is_ok());

    let (_, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::KeyNotFound as u32);
}

#[test]
fn Put_Get_Delete_Get_KeyNotFound() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);

    let key: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);
    let value: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);

    let status: u32 = common::put_key_value(*SERVER_ADDR, &Some(key.clone()), &Some(value.clone())).unwrap();
    assert_eq!(status, Status::Success as u32);

    let (retrived_value_opt, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::Success as u32);

    let retrived_value: Vec<u8> = retrived_value_opt.ok_or("GET failed. No value present").unwrap();
    assert_eq!(retrived_value, value);

    let (deleted_value_opt, status) = common::delete_key_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::Success as u32);

    let deleted_value: Vec<u8> = deleted_value_opt.ok_or("DELETE failed. No value present").unwrap();
    assert_eq!(deleted_value, value);

    let (_, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::KeyNotFound as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Put_Get_Delete_Delete_KeyNotFound() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);

    let key: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);
    let value: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);

    let status: u32 = common::put_key_value(*SERVER_ADDR, &Some(key.clone()), &Some(value.clone())).unwrap();
    assert_eq!(status, Status::Success as u32);

    let (retrived_value_opt, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::Success as u32);

    let retrived_value: Vec<u8> = retrived_value_opt.ok_or("GET failed. No value present").unwrap();
    assert_eq!(retrived_value, value);

    let (deleted_value_opt, status) = common::delete_key_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::Success as u32);

    let deleted_value: Vec<u8> = deleted_value_opt.ok_or("DELETE failed. No value present").unwrap();
    assert_eq!(deleted_value, value);

    let (_, status) = common::delete_key_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::KeyNotFound as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Put_Delete_Get_KeyNotFound() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);

    let key: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);
    let value: Vec<u8> = common::get_bytes(KEY_VALUE_SIZE_BYTES);

    let status: u32 = common::put_key_value(*SERVER_ADDR, &Some(key.clone()), &Some(value.clone())).unwrap();
    assert_eq!(status, Status::Success as u32);

    let (deleted_value_opt, status) = common::delete_key_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::Success as u32);

    let deleted_value: Vec<u8> = deleted_value_opt.ok_or("DELETE failed. No value present").unwrap();
    assert_eq!(deleted_value, value);

    let (_, status) = common::get_value(*SERVER_ADDR, &key).unwrap();
    assert_eq!(status, Status::KeyNotFound as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Undefined_Operation() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let proto_interface = common::get_proto_interface().unwrap();

    let mut request = Request::new();
    request.operation = u32::MAX;

    let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, *SERVER_ADDR).unwrap();
    let reply: Reply = extract_reply(&reply_msg).unwrap();
    assert_eq!(reply.status, Status::UndefinedOperation as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}
