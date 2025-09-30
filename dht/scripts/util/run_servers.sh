#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Check if an argument is provided and is a positive integer
if ! [[ $1 =~ ^[0-9]+$ ]] || [[ $1 -le 0 ]]; then
    echo "Usage: $0 <positive_integer> <server file path>.txt"
    exit 1
fi

# Check if argument is a text file path
if ! [[ $2 =~ ^.+\.txt$ ]]; then
    echo "Usage: $0 <positive_integer> <server file path>.txt"
    exit 2
fi

NUM_SERVERS="${1}"
SERVER_LIST_FILE="${2}"

# FIXME: This doesn't actually do any memory restrictions yet, see start_node.sh
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

        local PEERS=()
        for ((j=0; j<SERVERS_COUNT; j++)); do
            if [[ ${j} -ne ${i} ]]; then
                PEERS+=("${SERVERS[${j}]}")
            fi
        done

        local CONCATENATED_PEERS="$(IFS=,; echo "${PEERS[*]}")"
        ${SCRIPT_DIR}/start_node.sh ${PORT} "${CONCATENATED_PEERS}" ${VM_LIMIT_B}
    done
}

${SCRIPT_DIR}/generate_servers_list.sh "${NUM_SERVERS}" "${SERVER_LIST_FILE}"
start_servers $(realpath "${SCRIPT_DIR}/../../${SERVER_LIST_FILE}")
