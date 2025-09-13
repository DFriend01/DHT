#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
binary_name="dht"
server_pids=$(sudo netstat -tunlp | tail -n +3 | awk '{print $6}' | grep ${binary_name} | cut -d '/' -f1)

if [ -z "${server_pids}" ]; then
    echo "No servers found"
else
    nservers=$(echo -e ${server_pids} | wc -w)
    echo "Stopping ${nservers} servers with PIDs: ${server_pids}"
    kill ${server_pids}
fi
