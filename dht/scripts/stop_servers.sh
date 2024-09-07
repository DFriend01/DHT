#!/bin/bash
server_pids=$(ps -aux | grep dht | grep -v "grep" | awk '{print $2}')

if [ -z "${server_pids}" ]; then
    echo "No servers running"
else
    nservers=$(echo -e ${server_pids} | wc -w)
    echo "Stopping ${nservers} servers with PIDs: ${server_pids}"
    kill ${server_pids}
fi
