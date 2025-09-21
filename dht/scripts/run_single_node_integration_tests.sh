#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
TEST_ARGS="--show-output --test-threads 1"
export RUST_BACKTRACE=1

NUM_NODES="1"
SERVER_LIST_FILE="servers/single_node.txt"

# Ask for sudo permissions now to avoid asking later after the tests have finished
# since stop_servers.sh requires sudo permissions
sudo -v

${SCRIPT_DIR}/util/run_servers.sh "${NUM_NODES}" "${SERVER_LIST_FILE}"

cargo test --test test_single_node_is_alive -- ${TEST_ARGS} && \
    cargo test --test test_single_node_basic_operations -- ${TEST_ARGS} && \
    cargo test --test test_single_node_memory_capacity -- ${TEST_ARGS}

cargo test --test test_single_node_shutdown -- ${TEST_ARGS}

if [ $? -ne 0 ]; then
    echo "WARNING: The shutdown test failed. Killing server processes..."
    ${SCRIPT_DIR}/util/stop_servers.sh
fi
