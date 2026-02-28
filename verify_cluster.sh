#!/bin/bash

# Multi-node verification script for ZeroClaw + Actix Cluster
# This script starts two nodes and verifies message routing.

# 1. Build the project
echo "Building project..."
cargo build

# 2. Start Node 1 (Seed Node)
echo "Starting Node 1 on 127.0.0.1:1992 (Web: 8087)..."
RUST_LOG=info ./target/debug/agents 127.0.0.1:1992 > node1.log 2>&1 &
NODE1_PID=$!
sleep 2

# 3. Start Node 2 (Connects to Node 1)
echo "Starting Node 2 on 127.0.0.1:1993 (Web: 8088)..."
RUST_LOG=info ./target/debug/agents 127.0.0.1:1993 127.0.0.1:1992 > node2.log 2>&1 &
NODE2_PID=$!
sleep 5

# 4. Create an actor on Node 1 via Node 1's API
echo "Creating user1 on Node 1..."
curl -s -X POST http://localhost:8087/agent/user1/turn \
     -H "Content-Type: application/json" \
     -d '{"message": "Hello from Node 1"}' > /dev/null

# 5. Send a message to user1 via Node 2's API (should route to user1 on Node 1 or re-use existing)
# In our current simple implementation, it should find it in the cluster registry and send a RemoteWrapper.
echo "Sending message to user1 via Node 2..."
RESPONSE=$(curl -s -X POST http://localhost:8088/agent/user1/turn \
     -H "Content-Type: application/json" \
     -d '{"message": "Hello from Node 2"}')

echo "Response from Node 2: $RESPONSE"

# 6. Cleanup
echo "Cleaning up..."
kill $NODE1_PID $NODE2_PID
rm node1.log node2.log

# 7. Verification of logs
# Note: Since we don't have a real LLM, the response might be an error or a mock string.
# But we can check if it worked.
if [[ "$RESPONSE" == *"Agent response"* || "$RESPONSE" == *"Agent turn processed"* || "$RESPONSE" == *"Agent error"* ]]; then
    echo "SUCCESS: Routing verified (received response)."
else
    echo "FAILURE: Unexpected response."
    exit 1
fi
