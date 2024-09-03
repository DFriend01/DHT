#!/bin/bash
kill $(ps -aux | grep dht | grep -v "grep" | awk '{print $2}')
