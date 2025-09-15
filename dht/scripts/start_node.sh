#!/bin/bash

PORT=$1
ID=$2
VM_LIMIT_KB=$3

ulimit -v ${VM_LIMIT_KB}
cargo run --bin dht -- -p ${PORT} -s ${ID} 2>/dev/null 1>&2 &
echo "Started server on port ${PORT} with PID $!"
