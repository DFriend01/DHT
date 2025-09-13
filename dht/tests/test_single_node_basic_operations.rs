#![allow(non_snake_case)]

use dht::comm::proto::{extract_reply, Operation, Status};
use dht::comm::protogen::api::{Request, Reply};

mod common;
mod tests_prelude;

use tests_prelude::*;

#[ctor]
fn init() {
    common::init_logger();
}

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

#[test]
fn Put_Get_Success() {
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
fn Put_MissingKey() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let key: Option<Vec<u8>> = None;
    let value: Vec<u8> = common::get_rand_value();

    let status: u32 = common::put_key_value(*SERVER_ADDR, &key, &Some(value)).unwrap();
    assert_eq!(status, Status::MissingKey as u32);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);
}

#[test]
fn Put_MissingValue() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);
    let key: Vec<u8> = common::get_rand_key();
    let value: Option<Vec<u8>> = None;

    let status: u32 = common::put_key_value(*SERVER_ADDR, &Some(key), &value).unwrap();
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
    let key: Vec<u8> = common::get_rand_key();

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

    let (key, value, status) = common::put_rand_key_value(*SERVER_ADDR).unwrap();
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

    let (key, value, status) = common::put_rand_key_value(*SERVER_ADDR).unwrap();
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

    let (key, value, status) = common::put_rand_key_value(*SERVER_ADDR).unwrap();
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

    let (key, value, status) = common::put_rand_key_value(*SERVER_ADDR).unwrap();
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
