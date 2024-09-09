#![allow(non_snake_case)]

mod common;
mod tests_prelude;
use tests_prelude::*;

#[ctor]
fn init() {
    common::init_logger();
}

#[test]
fn Shutdown_Success() {
    let _ = common::ping_servers(vec![*SERVER_ADDR], true);
    let result = common::shutdown_servers(vec![*SERVER_ADDR], 5);
    assert!(result.is_ok());

    let result = common::ping_servers(vec![*SERVER_ADDR], false);
    assert!(result.is_err());
}
