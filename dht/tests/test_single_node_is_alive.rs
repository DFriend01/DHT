#![allow(non_snake_case)]

mod common;
mod tests_prelude;
use tests_prelude::*;

#[ctor]
fn init() {
    common::init_logger();
}

#[test]
fn Ping_Success() {
    let result = common::ping_servers(vec![*SERVER_ADDR], false);
    assert!(result.is_ok());
}
