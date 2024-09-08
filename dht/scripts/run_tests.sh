#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
TEST_ARGS="--show-output --test-threads 1"
export RUST_BACKTRACE=1

cargo test --test test_single_node_is_alive -- ${TEST_ARGS} && \
    cargo test --test test_single_node_basic_operations -- ${TEST_ARGS}

${SCRIPT_DIR}/stop_servers.sh
