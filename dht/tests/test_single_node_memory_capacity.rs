#![allow(non_snake_case)]

use dht::comm::proto::Status;
use std::time::{Duration, Instant};

use ntest::timeout;

mod common;
mod tests_prelude;

use tests_prelude::*;

#[ctor]
fn init() {
    common::init_logger();
}

#[test]
#[timeout(120000)]
fn test_memory_capacity() {
    let _result = common::ping_servers(vec![*SERVER_ADDR], true);

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);

    // Default memory capacity is 32MB defined in main.rs
    // This test assumes that the node runs with the default capacity
    const MEMORY_CAPACITY_BYTES: u64 = 32 * 1024 * 1024;
    const KEY_LEN: usize = 1024;
    const VALUE_LEN: usize = 8192;
    const PAYLOAD_SIZE: usize = KEY_LEN + VALUE_LEN;

    let mut total_inserted_size_bytes: u64 = 0;
    let mut latest_status: u32 = Status::Success as u32;
    let mut test_passed: bool = false;

    let mut now: Instant = Instant::now();
    let test_start_time = Instant::now();

    while latest_status != Status::OutOfMemory as u32 {
        if now.elapsed() > Duration::from_secs(10) {
            log::info!("Time elapsed: {}s, Total size of inserted data: {} Bytes",
                test_start_time.elapsed().as_secs(),
                total_inserted_size_bytes);
            now = Instant::now();
        }

        let key: Vec<u8> = common::get_bytes(KEY_LEN);
        let value: Vec<u8> = common::get_bytes(VALUE_LEN);

        latest_status = match common::put_key_value(*SERVER_ADDR, &Some(key), &Some(value)) {
            Ok(status) => {
                log::debug!("Received status code {}", status);
                status
            },
            Err(e) => {
                log::debug!("Failed to PUT key-value pair: {}", e);
                continue;
            }
        };

        match latest_status.try_into() {
            Ok(Status::Success) => {
                total_inserted_size_bytes += PAYLOAD_SIZE as u64;
                if total_inserted_size_bytes > MEMORY_CAPACITY_BYTES {
                    log::error!("Memory capacity exceeded. Inserted: {} Bytes, Max Size: {} Bytes",
                        total_inserted_size_bytes,
                        MEMORY_CAPACITY_BYTES);
                    break;
                }
            },
            Ok(Status::OutOfMemory) => {
                test_passed = true;
            },
            _ => {
                log::error!("Unexpected status code received: {}", latest_status);
            }
        };
    };

    let _ = common::wipe_servers(vec![*SERVER_ADDR], 1);

    if test_passed {
        let memory_utilization_ratio: f64 = (total_inserted_size_bytes as f64) / (MEMORY_CAPACITY_BYTES as f64);
        log::info!("Test completed in {}s with a memory utilization ratio of {:.1}",
            test_start_time.elapsed().as_secs(),
            memory_utilization_ratio);
    }

    assert!(test_passed);
}
