# CloakMesh — Complete Network Testing Guide

This guide is the definitive reference for testing every feature of the CloakMesh network.
It covers all CLI commands with copy-pasteable examples, expected outputs, and what to watch for
in node logs across all three layers of the stack.

---

## Prerequisites

```bash
# Build the Rust core
cd /workspaces/cloakmesh
cargo build -p cloakmesh-core

# Install Python dependencies (once)
cd orchestrator && poetry install && cd ..
```

---

## Starting the Three-Node Mesh

Open **four terminals** — three for nodes, one for the orchestrator CLI.

### Terminal 1 — Node Alpha (Bootstrap + Hosting Node)

```bash
cd /workspaces/cloakmesh/core
cargo run -- --port 4001 --id node-alpha
```

**Expected startup sequence:**
```
Config load warning: [9000] config: parse error ... — using defaults
[200] CloakMesh node starting, node_id: node-alpha, port: 4001, bootstrap_peers: 0
[200] Identity loaded, cloak_address: <ADDR>.cloak, pubkey: <hex>
[200] Node configuration validated, cell_size: 256, padding_enabled: true, default_hops: 3
[201] Starting decentralized bootstrapping via 0 directory authorities...
[200] gRPC server starting, addr: 0.0.0.0:4001
[200] Node fully initialized and listening
[202] Building new telescoping onion circuit, hops: 3
[202] Long-term guard nodes selected and pinned.
[202] Three-hop circuit established via telescoping construction, id: <uuid>
[200] SOCKS5 Proxy started. Visit .cloak addresses via this gateway. addr: 127.0.0.1:9050
```

📋 **Copy the `.cloak` address** — it's the `cloak_address:` value in the Identity loaded log line.

### Terminal 2 — Node Beta (Relay / Browser)

```bash
cd /workspaces/cloakmesh/core
cargo run -- --port 4002 --id node-beta --bootstrap 127.0.0.1:4001
```

**Expected — peer discovery happening:**
```
[200] CloakMesh node starting, node_id: node-beta, port: 4002, bootstrap_peers: 1
[201] Starting decentralized bootstrapping via 1 directory authorities...
[201] Bootstrap authority 127.0.0.1:4001 reachable (same identity or full bucket)
[201] Peer discovery via FindNode returned N candidates
[200] SOCKS5 Proxy started. addr: 127.0.0.1:9051
```

### Terminal 3 — Node Miner (ATK mining, knows both peers)

```bash
cd /workspaces/cloakmesh/core
cargo run -- --port 4003 --id node-miner --mine-atk \
  --bootstrap 127.0.0.1:4001 \
  --bootstrap 127.0.0.1:4002
```

**Expected:**
```
[200] Persistent Sled storage initialized for ATK mining
[201] Starting decentralized bootstrapping via 2 directory authorities...
[201] Bootstrap authority 127.0.0.1:4001 reachable
[201] Bootstrap authority 127.0.0.1:4002 reachable
[201] Peer discovery via FindNode returned N candidates
[200] SOCKS5 Proxy started. addr: 127.0.0.1:9052
```

---

## Section 1: Node Identity & Health

```bash
cd /workspaces/cloakmesh/orchestrator
```

### 1.1 — View your .cloak address
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli address
```
```
╭─ Your Node Identity ──────────────────────────────────────────────────────────╮
│ aedqpcdbqryj2lbj66mcup6relejktkzcp2kzt5fdz72tgvlpiemavqil5uq.cloak           │
╰────────────────────────────────────────────────────────────────────────────────╯
```

### 1.2 — Ping the node
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli ping
CLOAK_GRPC_PORT=4002 poetry run cloakcli ping
CLOAK_GRPC_PORT=4003 poetry run cloakcli ping
```
```
PONG node_id=0707886184709d2c... latency=1.4ms
```

### 1.3 — Node metrics
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli status
```
```
            Node Status
  Metric          │ Value
 ─────────────────┼──────────────────────────────────────
  Address         │ aedqpcdb...cloak
  Active Circuits │ 1
  Bytes Relayed   │ 0
  Reputation      │ 1.0
```

### 1.4 — Live dashboard
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli dash
# Press Ctrl+C to exit
```
Shows real-time: metrics panel, active circuits table, JSON log stream from core.

### 1.5 — Stream live telemetry
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli stream-metrics
# Press Ctrl+C to stop
```
```
uptime=42s circuits=1 dht_entries=3 relayed=0.0KB latency=0ms rep=1.000
uptime=44s circuits=1 dht_entries=3 relayed=0.0KB latency=0ms rep=1.000
```

---

## Section 2: DHT Peer Discovery

### 2.1 — View known relay peers
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli relays
CLOAK_GRPC_PORT=4002 poetry run cloakcli relays
```
```
╭─ Known DHT Relay Peers ──────────────────────────────────────╮
│ Node ID          │ Address           │ Reputation │ Latency  │
│ 0707886184709d2c │ 127.0.0.1:4001   │ 100        │ 0ms      │
```

### 2.2 — View active circuits
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli circuits
```
```
╭─ Active Onion Circuits ─────────────────────────────────╮
│ Circuit ID │ Hops │ Status │ Latency                    │
│ 91fcb75e   │ 3    │ READY  │ 0ms                        │
```

---

## Section 3: .cloak Web Hosting & Browsing

### 3.1 — Create a new .cloak site
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli site-init my_darknet_site
```
```
SUCCESS: Created beautiful static site in my_darknet_site/index.html
```

Inspect the generated site:
```bash
cat my_darknet_site/index.html
```

### 3.2 — Host the site on the mesh
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli host-static my_darknet_site 8080
```
```
SUCCESS: Started static server on port 8080 and bridged to aedqpcdb...cloak
Auto-publishing to DHT...
Publishing descriptor for aedqpcdb...cloak...
SUCCESS: Descriptor for aedqpcdb...cloak published successfully
Your site is LIVE! Browse it with: cloakcli browse aedqpcdb...cloak
```

**Watch Node Alpha's logs:**
```
[200] Hosting service on CloakMesh, port: 8080, addr: aedqpcdb...cloak
[200] Hosting aedqpcdb...cloak on local port 8080; published descriptor with intro_point 127.0.0.1:4001
```

### 3.3 — Fetch the DHT descriptor (verify it's published)
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli dht-fetch aedqpcdb...cloak
```
```
SUCCESS: Descriptor found
  Address: aedqpcdb...cloak
  Pubkey:  0707886184...
  Version: 1
  IntroPoint: 127.0.0.1:4001
```

### 3.4 — Browse from Node Beta (cross-node routing)
```bash
ADDR=aedqpcdb...cloak   # your actual address
CLOAK_GRPC_PORT=4002 poetry run cloakcli browse $ADDR
```

**Expected — full HTML response:**
```
Attempting to fetch index from aedqpcdb...cloak via Core SOCKS5 Proxy (127.0.0.1:9051)...

--- RESPONSE ---
HTTP/1.0 200 OK
Server: SimpleHTTP/0.6 Python/3.12.1
Content-type: text/html
Content-Length: 1154

<!DOCTYPE html>
<html lang="en">
<head>
    <title>CloakMesh Decentralized Site</title>
    ...
    <h1>Welcome to the Deep Web</h1>
    ...
</html>
----------------
```

**Watch Node Beta's logs:**
```
SOCKS5 CONNECT request received, target: aedqpcdb...cloak
Address not hosted locally. Initiating remote proxy via TunnelStream...
Found value at peer 127.0.0.1:4001
Routing to intro point 127.0.0.1:4001 for aedqpcdb...cloak
```

### 3.5 — Browse from Node Miner
```bash
CLOAK_GRPC_PORT=4003 poetry run cloakcli browse $ADDR
```

### 3.6 — Bridge an existing service (custom port)
```bash
# Start any HTTP service on port 9999 first
python3 -m http.server 9999 &

# Then bridge it to the mesh
CLOAK_GRPC_PORT=4001 poetry run cloakcli host $ADDR 9999
```

---

## Section 4: DHT Operations

### 4.1 — Manually publish a descriptor
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli dht-publish $ADDR
```
```
Publishing descriptor for aedqpcdb...cloak...
SUCCESS: Descriptor for aedqpcdb...cloak published successfully
```

### 4.2 — Fetch from a different node (cross-node DHT lookup)
```bash
CLOAK_GRPC_PORT=4002 poetry run cloakcli dht-fetch $ADDR
```
Node Beta will query its bootstrap peer (Alpha) via `FindValue` RPC and return the descriptor.

**Watch Node Beta's logs:**
```
Found value at peer 127.0.0.1:4001
```

---

## Section 5: Decentralized File Storage (AetherStore)

### 5.1 — Upload a text file
```bash
echo "Top secret document stored on the decentralized mesh." > secret.txt
CLOAK_GRPC_PORT=4001 poetry run cloakcli storage upload secret.txt
```
```
File Size: 55 bytes
SHA-256: a3f8d2c1...
Broadcasting File Manifest to network via Gossip...
Distributing data shards into DHT...
Upload Complete! File is decentralized.
To download, use: cloakcli storage download a3f8d2c1...
```

**Watch all three node terminals** — the gossip manifest propagates:
```
# Node Alpha
[202] GossipMessage ID: a3f8d2c1... from atk1_demo
[202] Gossip: saw file manifest hash=a3f8d2c1...
[202] Fanning out gossip a3f8d2c1... to N peers

# Node Beta (receives fan-out)
[202] GossipMessage ID: a3f8d2c1... from atk1_demo
[202] Gossip: saw file manifest hash=a3f8d2c1...
```

### 5.2 — Download from the same node
```bash
FILE_HASH=a3f8d2c1...    # use the hash printed by upload
CLOAK_GRPC_PORT=4001 poetry run cloakcli storage download $FILE_HASH --output recovered.txt
cat recovered.txt
```
```
Success! File reconstructed → recovered.txt (55 bytes)
Top secret document stored on the decentralized mesh.
```

### 5.3 — Download from a different node
```bash
CLOAK_GRPC_PORT=4002 poetry run cloakcli storage download $FILE_HASH -o from_beta.txt
cat from_beta.txt
```

### 5.4 — Upload a binary file
```bash
dd if=/dev/urandom bs=1024 count=8 of=test_binary.bin 2>/dev/null
CLOAK_GRPC_PORT=4001 poetry run cloakcli storage upload test_binary.bin
```

### 5.5 — Upload multiple files and verify persistence
```bash
for i in 1 2 3; do
  echo "Shard test document $i" > doc_$i.txt
  CLOAK_GRPC_PORT=4001 poetry run cloakcli storage upload doc_$i.txt
done
```

---

## Section 6: Messaging — Encrypted Chat

The `ChatStream` gRPC is a bidirectional streaming RPC. Messages are relayed back from the node.

### 6.1 — Send a chat message
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli chat-send "Hello from the darknet!" node-alpha
```
```
Sending chat message from 'node-alpha'...
Relay response from 'CloakMesh Relay': Hello from the darknet!
```

**Watch Node Alpha's logs:**
```
Received chat from node-alpha: Hello from the darknet!
```

### 6.2 — Open a listen session (Terminal A)
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli listen-chat
# Listening for incoming chat messages... Press Ctrl+C to stop.
```

### 6.3 — Send a message from another terminal (Terminal B)
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli chat-send "Incoming from node-beta" sender-beta
```
Terminal A displays:
```
CloakMesh Relay: Incoming from node-beta
```

### 6.4 — View chat history
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli view-chat-history
```
```
          Mesh Message History
 Timestamp        │ Sender    │ Message
 2026-05-21 14:22 │ Stanlley  │ Hello through the onion!
 2026-05-21 14:30 │ Relay-1   │ Awaiting handshake...
```

---

## Section 7: Encrypted File Transfer (gRPC Streaming)

`FileTransfer` is a client-streaming gRPC — files are chunked into 64KB pieces.

### 7.1 — Send a file
```bash
echo "Confidential: mesh transfer test" > transfer_test.txt
CLOAK_GRPC_PORT=4001 poetry run cloakcli file-send transfer_test.txt node-alpha
```
```
Sharing file 'transfer_test.txt' with node-alpha...
SUCCESS: File received: transfer_test.txt (35 bytes)
```

**Node Alpha logs:**
```
FileTransfer: received chunk 0 of transfer_test.txt (35 bytes)
FileTransfer complete: transfer_test.txt, total 35 bytes
```

### 7.2 — Send a larger file (multiple chunks)
```bash
dd if=/dev/urandom bs=1024 count=200 of=large_file.bin 2>/dev/null
CLOAK_GRPC_PORT=4001 poetry run cloakcli file-send large_file.bin node-alpha
```
This will stream in 64KB chunks (`chunk_index: 0, 1, 2, ...`).

### 7.3 — Listen for incoming files
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli file-recv
```
```
Waiting for incoming files...
Incoming file: secret.txt (35 bytes)
Saved to: ./downloads/secret.txt
```

### 7.4 — List received files
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli list-files
```
```
         Received Mesh Files
 Filename   │ Size  │ Origin         │ Status
 secret.txt │ 35B   │ ahqw6...cloak  │ DECRYPTED
 schema.pdf │ 1.2MB │ node-7...cloak │ READY
```

---

## Section 8: ATK Token Wallet

### 8.1 — Check balance
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli wallet balance
```
```
ATK Balance: 100.0 ATK
Derived from your Ed25519 identity key
```

### 8.2 — Transfer tokens
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli wallet transfer 15.5 atk1_recipient_address
```
```
Success! Sent 15.5 ATK to atk1_recipient_address
Transaction ID: mock-tx-1234
```

**Node Alpha logs:**
```
Received BroadcastTx for txid: mock-tx-1234
```

**Node Beta and Miner logs (gossip propagation):**
```
[202] GossipMessage ID: <hash> from <sender>
[202] Gossip: queuing tx mock-tx-1234 in mempool
[202] Fanning out gossip <hash> to N peers
```

### 8.3 — Verify gossip reaches all nodes
```bash
# Transfer and watch all terminals simultaneously
CLOAK_GRPC_PORT=4001 poetry run cloakcli wallet transfer 1.0 atk1_test
# Should appear in node-beta and node-miner terminal logs within ~1 second
```

---

## Section 9: Capability Tokens (Access Control)

### 9.1 — Issue a read token
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli auth-issue $ADDR read 3600
```
```json
{"token_id":"a3f8d2c1b4e5f6a7","cloak_address":"aedqpcdb...cloak","scope":"read","expires_at":1748254800,"signature":""}
```

### 9.2 — Issue a write token with custom TTL
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli auth-issue $ADDR write 7200
```

### 9.3 — Verify a token
```bash
TOKEN=$(CLOAK_GRPC_PORT=4001 poetry run cloakcli auth-issue $ADDR read 3600 2>/dev/null | tail -1)
CLOAK_GRPC_PORT=4001 poetry run cloakcli auth-verify "$TOKEN"
```
```
╭─ Capability Token ─────────────────────────────────╮
│ Field      │ Value                                  │
│ Token ID   │ a3f8d2c1b4e5f6a7                      │
│ Address    │ aedqpcdb...cloak                       │
│ Scope      │ read                                   │
│ Expires At │ 1748254800                             │
│ Status     │ VALID                                  │
╰────────────────────────────────────────────────────╯
```

### 9.4 — Verify via gRPC (on the Rust node)
```bash
# The CapabilityService.Verify RPC validates token_id, scope, expiry, and signature
# Use the grpc_client directly:
python3 -c "
from cloakcli.api.grpc_client import CloakGrpcClient
from proto import capability_pb2
c = CloakGrpcClient(port=4001)
# verification via gRPC
c.close()
"
```

---

## Section 10: Background Node Management

### 10.1 — Start a node in the background
```bash
cd /workspaces/cloakmesh/core   # NodeManager runs the binary from here
CLOAK_GRPC_PORT=4004 poetry run cloakcli node-start --port 4004 --id node-delta --bootstrap 127.0.0.1:4001
```
```
Node started on port 4004 (pid 12345)
Node started on gRPC port 4004 | SOCKS5 on 9053
```

### 10.2 — Verify it's running
```bash
CLOAK_GRPC_PORT=4004 poetry run cloakcli ping
```

### 10.3 — Stop it
```bash
CLOAK_GRPC_PORT=4004 poetry run cloakcli node-stop --port 4004
```
```
Node 12345 stopped
```

---

## Section 11: Interactive Console

The interactive REPL gives access to all commands in one session:

```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli interactive
```

Inside the console:
```
cloak> help          # Show all commands
cloak> status        # Node metrics
cloak> ping          # Keepalive ping
cloak> relays        # Known peers
cloak> circuits      # Active circuits
cloak> dht-fetch     # Prompt for address
cloak> host-static   # Prompt for dir + port
cloak> browse        # Prompt for .cloak address
cloak> chat          # Prompt for target and message
cloak> listen        # Start message listener
cloak> history       # Chat history
cloak> file-send     # Prompt for path and target
cloak> file-recv     # Receive files
cloak> list-files    # Received files list
cloak> wallet balance
cloak> wallet transfer 5.0 atk1_addr
cloak> storage upload secret.txt
cloak> storage download <hash>
cloak> auth          # Issue capability token
cloak> node-start    # (direct commands from interactive mode)
cloak> clear         # Clear screen
cloak> exit          # Quit
```

---

## Section 12: Full Integration Test Script

Run this to validate the entire stack in sequence:

```bash
#!/bin/bash
set -e
cd /workspaces/cloakmesh/orchestrator

ADDR=$(CLOAK_GRPC_PORT=4001 poetry run cloakcli address 2>/dev/null | grep -oP '[a-z2-7]{60,}\.cloak' | head -1)
echo "=== CloakMesh Integration Test ==="
echo "Node address: $ADDR"

echo -e "\n[1/10] Ping all nodes..."
for PORT in 4001 4002 4003; do
  CLOAK_GRPC_PORT=$PORT poetry run cloakcli ping 2>/dev/null
done

echo -e "\n[2/10] Host site on Alpha..."
CLOAK_GRPC_PORT=4001 poetry run cloakcli host-static my_darknet_site 8080

echo -e "\n[3/10] Verify DHT descriptor..."
CLOAK_GRPC_PORT=4001 poetry run cloakcli dht-fetch $ADDR

echo -e "\n[4/10] Browse from Beta..."
CLOAK_GRPC_PORT=4002 poetry run cloakcli browse $ADDR

echo -e "\n[5/10] Browse from Miner..."
CLOAK_GRPC_PORT=4003 poetry run cloakcli browse $ADDR

echo -e "\n[6/10] Upload file to DHT..."
echo "Mesh storage test $(date)" > /tmp/mesh_test.txt
CLOAK_GRPC_PORT=4001 poetry run cloakcli storage upload /tmp/mesh_test.txt
FILE_HASH=$(CLOAK_GRPC_PORT=4001 poetry run cloakcli storage upload /tmp/mesh_test.txt 2>/dev/null | grep -oP '[a-f0-9]{64}' | head -1)

echo -e "\n[7/10] Send ATK tokens..."
CLOAK_GRPC_PORT=4001 poetry run cloakcli wallet balance
CLOAK_GRPC_PORT=4001 poetry run cloakcli wallet transfer 5.0 atk1_integration_test

echo -e "\n[8/10] Chat message..."
CLOAK_GRPC_PORT=4001 poetry run cloakcli chat-send "Integration test $(date)" test-runner

echo -e "\n[9/10] Issue capability token..."
TOKEN=$(CLOAK_GRPC_PORT=4001 poetry run cloakcli auth-issue $ADDR read 3600 2>/dev/null | tail -1)
CLOAK_GRPC_PORT=4001 poetry run cloakcli auth-verify "$TOKEN"

echo -e "\n[10/10] Relay peer list..."
CLOAK_GRPC_PORT=4002 poetry run cloakcli relays

echo -e "\n=== ALL TESTS PASSED ==="
```

---

## Port & Environment Reference

| Node | gRPC Port | SOCKS5 Proxy | `CLOAK_GRPC_PORT` | PID file |
|------|-----------|--------------|-------------------|----------|
| node-alpha | 4001 | 127.0.0.1:9050 | 4001 | /tmp/cloakmesh/node-4001.pid |
| node-beta  | 4002 | 127.0.0.1:9051 | 4002 | /tmp/cloakmesh/node-4002.pid |
| node-miner | 4003 | 127.0.0.1:9052 | 4003 | /tmp/cloakmesh/node-4003.pid |
| node-delta | 4004 | 127.0.0.1:9053 | 4004 | /tmp/cloakmesh/node-4004.pid |

**SOCKS5 port formula:** `SOCKS5 = gRPC_port + 5049`

**Using cURL through SOCKS5:**
```bash
curl --socks5-hostname 127.0.0.1:9051 http://aedqpcdb...cloak/
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `Failed to locate address in distributed DHT` | Site not hosted or node not bootstrapped | Run `host-static` first; wait for `published descriptor` log |
| `Connection reset by peer` | TunnelStream intro_point unreachable | Verify Alpha running on correct port |
| `Got unexpected extra argument` in download | Wrong syntax | Use `--output file.txt` not positional arg |
| Other nodes have no gossip logs | Not in each other's routing tables | Use `--bootstrap` to multiple peers |
| Same `.cloak` address on all nodes | Stale shared identity key | `rm -rf data_node-*` then restart |
| `No relay peers known yet` | `relays` called before bootstrap completes | Wait ~1s after node start for bootstrap to complete |
| `PONG` shows wrong node_id | `CLOAK_GRPC_PORT` pointing to wrong node | Double-check env var matches the terminal's node port |
| `Stream error` in `stream-metrics` | gRPC streaming not yet implemented | Use `status` or `dash` instead for polling |
