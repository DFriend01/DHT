#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

VM_LIMIT_MB=64
VM_LIMIT_B=$((VM_LIMIT_MB * 1024 * 1024))

function start_servers() {
    local SERVERS_FILE=$1
    local SERVERS_COUNT=$(wc -l < $SERVERS_FILE)
    local SERVERS=($(cat $SERVERS_FILE))

    echo "Starting ${SERVERS_COUNT} servers"
    for ((i=0; i<SERVERS_COUNT; i++)); do
        local SERVER=${SERVERS[$i]}
        local PORT=$(echo $SERVER | cut -d':' -f2)
        ${SCRIPT_DIR}/start_node.sh ${PORT} ${i} ${VM_LIMIT_B}
    done
}

${SCRIPT_DIR}/generate_servers_list.sh 1 servers/single_server.txt
start_servers $(realpath ${SCRIPT_DIR}/../servers/single_server.txt)
