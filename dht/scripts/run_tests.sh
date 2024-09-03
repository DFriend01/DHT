#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
export RUST_BACKTRACE=1
cargo test -- --show-output --test-threads 1
${SCRIPT_DIR}/stop_servers.sh
