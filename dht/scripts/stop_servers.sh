#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
binary_path=$(realpath -m ${SCRIPT_DIR}/../target/debug/dht)
server_pids=$(ps -aux | grep ${binary_path} | grep -v "grep" | awk '{print $2}')

if [ -z "${server_pids}" ]; then
    echo "No servers running"
else
    nservers=$(echo -e ${server_pids} | wc -w)
    echo "Stopping ${nservers} servers with PIDs: ${server_pids}"
    kill ${server_pids}
fi
