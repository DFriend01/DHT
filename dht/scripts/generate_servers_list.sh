#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Check if an argument is provided and is a positive integer
if [[ $# -ne 1 ]] || ! [[ $1 =~ ^[0-9]+$ ]] || [[ $1 -le 0 ]]; then
    echo "Usage: $0 <positive_integer>"
    exit 1
fi

N=$1
OUTPUT_FILE="${SCRIPT_DIR}/../servers.txt"
PORTS_FOUND=0
START_PORT=1024
UPPER_BOUND=65535
AVAILABLE_PORTS=()

# Function to check if a port is available
is_port_available() {
    local port=$1
    if ss -tuln | grep -q ":$port "; then
        return 1
    else
        return 0
    fi
}

# Find N available ports
while [[ $PORTS_FOUND -lt $N ]]; do
    if [[ $START_PORT -gt $UPPER_BOUND ]]; then
        echo "Reached upper bound of port range without finding $N available ports."
        exit 2
    fi

    if is_port_available $START_PORT; then
        AVAILABLE_PORTS+=($START_PORT)
        PORTS_FOUND=$((PORTS_FOUND + 1))
    fi
  START_PORT=$((START_PORT + 1))
done

# Write the available ports to the output file
OUT=""
for port in "${AVAILABLE_PORTS[@]}"; do
    ADDRESS="127.0.0.1:${port}"
    OUT+="${ADDRESS}\n"
done

echo -en `trim_newlines $OUT` > $OUTPUT_FILE
echo "Found $N available ports and wrote them to `realpath $OUTPUT_FILE`"
