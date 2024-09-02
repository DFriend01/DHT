#!/bin/bash
set -e

function start_servers() {
    local SERVERS_FILE=$1
    local SERVERS_COUNT=$(wc -l < $SERVERS_FILE)
    local SERVERS=($(cat $SERVERS_FILE))

    for ((i=0; i<SERVERS_COUNT; i++)); do
        local SERVER=${SERVERS[$i]}
        local PORT=$(echo $SERVER | cut -d':' -f2)
        cargo run --bin dht -- -p $PORT -s $i 2>/dev/null 1>&2 &
    done
}

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Generate server lists
${SCRIPT_DIR}/generate_servers_list.sh 1 servers/single_server.txt

# Start server
start_servers $(realpath ${SCRIPT_DIR}/../servers/single_server.txt)
sleep 1

cargo test -- --show-output --test-threads 1
