#!/bin/bash

PORT=$1
ID=$2
VM_LIMIT_B=$3

# TODO: Enforce rules on memory usage robustly, ulimit does not work as expected
# ulimit -v ${VM_LIMIT_B}
cargo run --bin dht -- -p ${PORT} -s ${ID} -l info 2>/dev/null 1>&2 &
echo "Started server on port ${PORT} with PID $!"
