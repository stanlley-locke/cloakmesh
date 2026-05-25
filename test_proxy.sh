#!/bin/bash
cd core
cargo run --release -- --port 4001 --id node-1 > node1.log 2>&1 &
PID1=$!
cargo run --release -- --port 4002 --id node-2 > node2.log 2>&1 &
PID2=$!
cargo run --release -- --port 4003 --id node-3 > node3.log 2>&1 &
PID3=$!
sleep 5
echo "Nodes started."
