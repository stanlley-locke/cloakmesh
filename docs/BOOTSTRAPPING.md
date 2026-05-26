# CloakMesh Bootstrapping Guide

Complete walkthrough for setting up a CloakMesh network from scratch — from launching the first authority node to connecting peers across the internet using IP addresses, domain names, or GitHub Codespaces public URLs.

---

## Table of Contents

- [How Bootstrapping Works](#how-bootstrapping-works)
- [Bootstrap Address Formats](#bootstrap-address-formats)
- [Part 1: Setting Up the Alpha Node (Bootstrap Authority)](#part-1-setting-up-the-alpha-node-bootstrap-authority)
- [Part 2: Connecting a Second Node (Local)](#part-2-connecting-a-second-node-local)
- [Part 3: Running a Full Local Three-Node Mesh](#part-3-running-a-full-local-three-node-mesh)
- [Part 4: Connecting Across the Internet (VPS / Public IP)](#part-4-connecting-across-the-internet-vps--public-ip)
- [Part 5: Connecting via GitHub Codespaces](#part-5-connecting-via-github-codespaces)
- [Part 6: Using a JSON Peers File](#part-6-using-a-json-peers-file)
- [Part 7: Background Node Management (Python CLI)](#part-7-background-node-management-python-cli)
- [Part 8: Verifying the Bootstrap Succeeded](#part-8-verifying-the-bootstrap-succeeded)
- [Part 9: Persistent Bootstrap with systemd](#part-9-persistent-bootstrap-with-systemd)
- [Troubleshooting](#troubleshooting)
- [Bootstrap Architecture Reference](#bootstrap-architecture-reference)

---

## How Bootstrapping Works

When a CloakMesh node starts, it knows nothing about the network. Bootstrapping is the process of joining:

```
New Node (node-beta)
    │
    │  1. Connect to each --bootstrap address
    ▼
Bootstrap Authority (node-alpha)
    │  ← KeepAlive(Ping)  →  Pong{node_id: hex(pubkey)}
    │     α added to β's routing table
    │
    │  2. β sends FindNode{target_id: own_pubkey}
    ▼
α responds with K closest peers it knows
    │
    │  3. β adds discovered peers to its routing table
    │     β dials each discovered peer
    │     β sends FindNode again (iterative walk)
    ▼
β is now part of the DHT mesh
```

**Three phases of bootstrap:**

| Phase | Action | Log line |
|-------|--------|---------|
| **Ping** | Confirm authority is reachable | `[201] Bootstrap authority 127.0.0.1:4001 reachable` |
| **FindNode** | Ask authority for peers closest to our own key | `[201] Peer discovery via FindNode returned N candidates` |
| **Propagation** | Contact discovered peers, repeat FindNode | `[201] Discovered peer <id> at <address>` |

The bootstrap is complete when the routing table has enough peers to form circuits. With 3+ nodes, a full 3-hop circuit can be built.

---

## Bootstrap Address Formats

CloakMesh accepts bootstrap addresses in four formats. All are normalized internally to `host:port`.

| Format | Example | When to use |
|--------|---------|-------------|
| **Plain `host:port`** | `127.0.0.1:4001` | Local dev, VPS with known IP |
| **Domain with port** | `peer.example.com:4001` | Domain-named VPS |
| **HTTPS URL with explicit port** | `https://peer.example.com:4001/` | Reverse-proxied deployments |
| **GitHub Codespaces URL** | `https://ideal-orbit-xyz-4001.app.github.dev/` | Codespaces (port in subdomain) |

**GitHub Codespaces URL parsing:**

The port is embedded in the subdomain label: `<workspace-name>-`**`PORT`**`.app.github.dev`

```
https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/
                                    ^^^^
                              gRPC port extracted here
```

CloakMesh reads this as `ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev:4001`.

---

## Part 1: Setting Up the Alpha Node (Bootstrap Authority)

The alpha node is the first node — it has no bootstrap peers itself. It is the entry point that all subsequent nodes will contact.

### Step 1: Build the binary

```bash
cd /workspaces/cloakmesh
cargo build --release -p cloakmesh-core
# Binary: ./target/release/cloakmesh
```

### Step 2: Start the alpha node

```bash
cd /workspaces/cloakmesh/core

cargo run --release -- \
  --port 4001 \
  --id node-alpha
```

Or using the release binary directly:

```bash
./target/release/cloakmesh --port 4001 --id node-alpha
```

**Expected startup logs:**

```
Config load warning: [9000] config: parse error ... — using defaults
[200] CloakMesh node starting, node_id: node-alpha, port: 4001, bootstrap_peers: 0, public_addr: 127.0.0.1:4001
[200] Identity loaded, cloak_address: aedqpcdb...cloak, pubkey: 0707886184...
[200] Node configuration validated, cell_size: 256, padding_enabled: true, default_hops: 3, pq_hybrid: true
[200] gRPC server starting, addr: 0.0.0.0:4001
[200] Node fully initialized and listening
[202] Building new telescoping onion circuit, hops: 3
[202] Three-hop circuit established via telescoping construction, id: <uuid>
[200] SOCKS5 Proxy started. addr: 127.0.0.1:9050
```

### Step 3: Note the .cloak address

Copy the address from the identity line — you'll need it to publish descriptors:

```
cloak_address: aedqpcdbqryj2lbj66mcup6relejktkzcp2kzt5fdz72tgvlpiemavqil5uq.cloak
```

### Step 4: Verify alpha is up

In a new terminal:

```bash
cd /workspaces/cloakmesh/orchestrator
CLOAK_GRPC_PORT=4001 poetry run cloakcli ping
```

```
PONG node_id=0707886184709d2c... latency=1.2ms
```

---

## Part 2: Connecting a Second Node (Local)

With alpha running, beta can bootstrap to it.

### Start node-beta

```bash
cd /workspaces/cloakmesh/core

cargo run --release -- \
  --port 4002 \
  --id node-beta \
  --bootstrap 127.0.0.1:4001
```

**Expected logs:**

```
[200] CloakMesh node starting, node_id: node-beta, port: 4002, bootstrap_peers: 1, public_addr: 127.0.0.1:4002
[200] Bootstrap peers: ["127.0.0.1:4001"]
[201] Starting decentralized bootstrapping via 1 directory authorities...
[201] Bootstrap authority 127.0.0.1:4001 reachable (same identity or full bucket)
[201] Peer discovery via FindNode returned 1 candidates
[201] Discovered peer 0707886... at 127.0.0.1:4001
[200] SOCKS5 Proxy started. addr: 127.0.0.1:9051
```

**Verify cross-node routing:**

```bash
# Alpha should now know about beta (and vice versa)
CLOAK_GRPC_PORT=4001 poetry run cloakcli relays
CLOAK_GRPC_PORT=4002 poetry run cloakcli relays
```

---

## Part 3: Running a Full Local Three-Node Mesh

Three nodes gives you a full 3-hop onion circuit (Guard → Middle → Exit).

### Terminal 1 — Bootstrap authority

```bash
cargo run --release -- --port 4001 --id node-alpha
```

### Terminal 2 — Relay node

```bash
cargo run --release -- \
  --port 4002 \
  --id node-beta \
  --bootstrap 127.0.0.1:4001
```

### Terminal 3 — Mining node (bootstraps to BOTH)

```bash
cargo run --release -- \
  --port 4003 \
  --id node-miner \
  --mine-atk \
  --bootstrap 127.0.0.1:4001 \
  --bootstrap 127.0.0.1:4002
```

> **Why bootstrap to multiple peers?** Bootstrapping to both alpha and beta gives node-miner a richer initial routing table. It will also discover beta-as-peer when it sends `FindNode` to alpha, but listing beta explicitly ensures it's added immediately.

### Verify the full mesh

```bash
# All three nodes should see each other as relay peers
CLOAK_GRPC_PORT=4001 poetry run cloakcli relays   # sees beta, miner
CLOAK_GRPC_PORT=4002 poetry run cloakcli relays   # sees alpha, miner
CLOAK_GRPC_PORT=4003 poetry run cloakcli relays   # sees alpha, beta

# All three have circuits
CLOAK_GRPC_PORT=4001 poetry run cloakcli circuits
CLOAK_GRPC_PORT=4002 poetry run cloakcli circuits
CLOAK_GRPC_PORT=4003 poetry run cloakcli circuits
```

### Port reference

| Node | gRPC | SOCKS5 | `CLOAK_GRPC_PORT` |
|------|------|--------|-------------------|
| node-alpha | 4001 | 9050 | 4001 |
| node-beta | 4002 | 9051 | 4002 |
| node-miner | 4003 | 9052 | 4003 |
| node-delta | 4004 | 9053 | 4004 |

---

## Part 4: Connecting Across the Internet (VPS / Public IP)

To build a real multi-machine mesh, alpha runs on a VPS with a public IP, and beta connects from another machine.

### On the VPS (alpha)

```bash
# Open the gRPC port in your firewall
ufw allow 4001/tcp comment "CloakMesh gRPC"

# Start the node — announce your public IP
./cloakmesh \
  --port 4001 \
  --id node-alpha-vps \
  --public-addr 203.0.113.10:4001
```

The `--public-addr` flag tells alpha to announce `203.0.113.10:4001` to other peers when they ask "who is this node?" This address goes into `CloakDescriptor.intro_points` and `FindNode` responses.

### On another machine (beta)

```bash
./cloakmesh \
  --port 4001 \
  --id node-beta-home \
  --bootstrap 203.0.113.10:4001
```

### On yet another machine (gamma — knows both)

```bash
./cloakmesh \
  --port 4001 \
  --id node-gamma \
  --bootstrap 203.0.113.10:4001 \
  --bootstrap home-machine.example.com:4001
```

### With a domain name

If your VPS has a DNS record (`bootstrap.example.com → 203.0.113.10`):

```bash
# beta connects via domain name
./cloakmesh --port 4001 --id node-beta --bootstrap bootstrap.example.com:4001
```

---

## Part 5: Connecting via GitHub Codespaces

GitHub Codespaces forwards ports through HTTPS. When you expose port 4001 in Codespaces, it gets a public URL like:

```
https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/
```

### Step 1: Start alpha in Codespaces

```bash
cd /workspaces/cloakmesh/core
cargo run --release -- --port 4001 --id codespace-alpha
```

### Step 2: Expose port 4001 in Codespaces

In VS Code: **Ports** panel → **Forward a Port** → enter `4001` → set visibility to **Public**.

Your public URL appears in the Ports panel, e.g.:
```
https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/
```

### Step 3: Tell alpha its public address

Restart with `--public-addr` so descriptors advertise the Codespaces URL:

```bash
cargo run --release -- \
  --port 4001 \
  --id codespace-alpha \
  --public-addr https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/
```

CloakMesh normalizes this to `ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev:4001`.

### Step 4: Connect from anywhere

Any machine on the internet:

```bash
./cloakmesh \
  --port 4001 \
  --id remote-node \
  --bootstrap https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/
```

Or a second Codespace:

```bash
cargo run --release -- \
  --port 4002 \
  --id codespace-beta \
  --bootstrap https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/
```

### Step 5: Expose beta's port too (optional)

If beta should also be reachable externally, expose port 4002 in Codespaces and start with:

```bash
cargo run --release -- \
  --port 4002 \
  --id codespace-beta \
  --bootstrap https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/ \
  --public-addr https://ideal-orbit-pjgjrvxgg7wjhw6-4002.app.github.dev/
```

### Using the Python CLI for Codespaces bootstrap

```bash
cd /workspaces/cloakmesh/orchestrator

CLOAK_GRPC_PORT=4002 poetry run cloakcli node-start \
  --port 4002 \
  --id codespace-beta \
  --bootstrap "https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/" \
  --public-addr "https://ideal-orbit-pjgjrvxgg7wjhw6-4002.app.github.dev/"
```

---

## Part 6: Using a JSON Peers File

For larger networks or reproducible setups, maintain a `peers.json` file.

### File formats

**Format A — Simple array:**

```json
[
  "127.0.0.1:4001",
  "https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/",
  "peer.example.com:4001"
]
```

**Format B — Rich object (recommended):**

```json
{
  "_format": "v1",
  "network": "cloakmesh-devnet",
  "peers": [
    {
      "address": "127.0.0.1:4001",
      "id": "node-alpha",
      "note": "Local bootstrap authority"
    },
    {
      "address": "https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/",
      "id": "codespace-alpha",
      "note": "GitHub Codespaces node"
    },
    {
      "address": "203.0.113.10:4001",
      "id": "vps-relay",
      "note": "Production relay on VPS"
    }
  ]
}
```

Both formats support mixing plain `host:port`, HTTPS URLs, and Codespaces URLs.

### Use the peers file

**With the Rust binary:**

```bash
./cloakmesh \
  --port 4002 \
  --id node-beta \
  --peers-file /path/to/peers.json
```

**With the Python CLI:**

```bash
CLOAK_GRPC_PORT=4002 poetry run cloakcli node-start \
  --port 4002 \
  --id node-beta \
  --peers-file peers.json
```

**Combine with additional `--bootstrap` flags:**

```bash
# peers.json provides the base list; extra --bootstrap adds more
./cloakmesh \
  --port 4002 \
  --peers-file peers.json \
  --bootstrap extra-peer.example.com:4001
```

All sources are merged and deduplicated.

### Generate a peers file with the CLI

```bash
cd /workspaces/cloakmesh/orchestrator

poetry run cloakcli peers-gen \
  "127.0.0.1:4001" \
  "https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/" \
  "203.0.113.10:4001" \
  --output peers.json \
  --network cloakmesh-devnet
```

```
Peers file written: peers.json (3 peers)
```

### Distribute the peers file

```bash
# Share via HTTP
python3 -m http.server 8000 -d /path/to/configs/

# Other nodes can reference it directly (future: URL peers-file support)
```

---

## Part 7: Background Node Management (Python CLI)

Use the Python CLI to manage nodes as background processes without terminal occupation.

### Start nodes

```bash
cd /workspaces/cloakmesh/orchestrator

# Start alpha (no bootstrap, is the authority)
poetry run cloakcli node-start \
  --port 4001 \
  --id node-alpha

# Start beta (bootstraps to alpha)
poetry run cloakcli node-start \
  --port 4002 \
  --id node-beta \
  --bootstrap 127.0.0.1:4001

# Start miner (bootstraps to both, with persistence)
poetry run cloakcli node-start \
  --port 4003 \
  --id node-miner \
  --bootstrap 127.0.0.1:4001 \
  --bootstrap 127.0.0.1:4002 \
  --mine-atk

# Start a node with a peers file and public Codespaces address
poetry run cloakcli node-start \
  --port 4004 \
  --id node-delta \
  --peers-file peers.json \
  --public-addr "https://ideal-orbit-pjgjrvxgg7wjhw6-4004.app.github.dev/"
```

### List and inspect running nodes

```bash
# See all running nodes
poetry run cloakcli node-list
```

```
╭─ Running CloakMesh Nodes ────────────────────────────────╮
│ Port │ SOCKS5 │ PID    │ Status                          │
│ 4001 │ 9050   │ 12345  │ RUNNING                         │
│ 4002 │ 9051   │ 12346  │ RUNNING                         │
│ 4003 │ 9052   │ 12347  │ RUNNING                         │
╰──────────────────────────────────────────────────────────╯
```

```bash
# Check one node
poetry run cloakcli node-status --port 4001
```

```
RUNNING port=4001 pid=12345 log=/tmp/cloakmesh/logs/node-4001.log
```

```bash
# Tail the node log
tail -f /tmp/cloakmesh/logs/node-4001.log
```

### Stop nodes

```bash
poetry run cloakcli node-stop --port 4003
poetry run cloakcli node-stop --port 4002
poetry run cloakcli node-stop --port 4001
```

---

## Part 8: Verifying the Bootstrap Succeeded

Run all of these after a bootstrap to confirm a healthy mesh.

### 1. Ping each node

```bash
for PORT in 4001 4002 4003; do
  CLOAK_GRPC_PORT=$PORT poetry run cloakcli ping
done
```

```
PONG node_id=0707886184709d2c... latency=1.2ms
PONG node_id=9a3f8b2e1c0d7f4a... latency=1.8ms
PONG node_id=b4e5d6c7a8f92031... latency=0.9ms
```

### 2. Check relay tables (peers discovered)

```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli relays
```

```
╭─ Known DHT Relay Peers ──────────────────────────────────────╮
│ Node ID          │ Address          │ Reputation │ Latency   │
│ 9a3f8b2e1c0d7f4a │ 127.0.0.1:4002  │ 100        │ 1ms       │
│ b4e5d6c7a8f92031 │ 127.0.0.1:4003  │ 100        │ 0ms       │
╰──────────────────────────────────────────────────────────────╯
```

If a node has 0 relays, bootstrap didn't complete — see [Troubleshooting](#troubleshooting).

### 3. Check active circuits

```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli circuits
```

```
╭─ Active Onion Circuits ──────────────────────────────────────╮
│ Circuit ID       │ Hops │ Status  │ Latency                  │
│ 91fcb75e-fae4... │ 3    │ READY   │ 2ms                      │
╰──────────────────────────────────────────────────────────────╯
```

### 4. Test cross-node DHT

```bash
# Publish a descriptor from alpha
CLOAK_GRPC_PORT=4001 poetry run cloakcli dht-publish $(CLOAK_GRPC_PORT=4001 poetry run cloakcli address 2>/dev/null | grep -oP '[a-z2-7]{60,}\.cloak')

# Fetch it from beta (cross-node DHT lookup)
ADDR=$(CLOAK_GRPC_PORT=4001 poetry run cloakcli address 2>/dev/null | grep -oP '[a-z2-7]{60,}\.cloak')
CLOAK_GRPC_PORT=4002 poetry run cloakcli dht-fetch $ADDR
```

### 5. Test cross-node browsing

```bash
# Host a site on alpha
CLOAK_GRPC_PORT=4001 poetry run cloakcli host-static my_darknet_site 8080

# Browse it from beta (routed through the mesh)
ADDR=$(CLOAK_GRPC_PORT=4001 poetry run cloakcli address 2>/dev/null | grep -oP '[a-z2-7]{60,}\.cloak')
CLOAK_GRPC_PORT=4002 poetry run cloakcli browse $ADDR
```

### 6. Test gossip propagation

```bash
# Upload a file from alpha — should gossip to all nodes
CLOAK_GRPC_PORT=4001 poetry run cloakcli storage upload /etc/hostname

# Check all three terminals for gossip log lines like:
# [202] GossipMessage ID: abc123... from atk1_demo
# [202] Fanning out gossip abc123... to 2 peers
```

### 7. Check live metrics

```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli dash
```

The dashboard shows real-time: peers, circuits, DHT entries, bytes relayed.

---

## Part 9: Persistent Bootstrap with systemd

For a production VPS that should always be available as a bootstrap authority.

### 1. Install the binary

```bash
sudo cp ./target/release/cloakmesh /usr/local/bin/cloakmesh
sudo useradd -r -s /sbin/nologin cloakmesh
sudo mkdir -p /var/lib/cloakmesh
sudo chown cloakmesh:cloakmesh /var/lib/cloakmesh
```

### 2. Create the service unit

`/etc/systemd/system/cloakmesh-alpha.service`:

```ini
[Unit]
Description=CloakMesh Bootstrap Node (Alpha)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=cloakmesh
WorkingDirectory=/var/lib/cloakmesh

ExecStart=/usr/local/bin/cloakmesh \
    --port 4001 \
    --id node-alpha-prod \
    --mine-atk \
    --public-addr 203.0.113.10:4001 \
    --peers-file /etc/cloakmesh/peers.json

Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/var/lib/cloakmesh

[Install]
WantedBy=multi-user.target
```

### 3. Create a peers file for the authority

```bash
sudo mkdir -p /etc/cloakmesh
sudo tee /etc/cloakmesh/peers.json > /dev/null << 'EOF'
{
  "_format": "v1",
  "network": "cloakmesh-mainnet",
  "peers": []
}
EOF
```

The authority itself has no bootstrap peers — it IS the bootstrap peer. Other nodes add themselves to the peers file over time.

### 4. Enable and start

```bash
sudo systemctl daemon-reload
sudo systemctl enable cloakmesh-alpha
sudo systemctl start cloakmesh-alpha

# Monitor
sudo journalctl -u cloakmesh-alpha -f
```

### 5. Check it's reachable

```bash
# From another machine
CLOAK_GRPC_PORT=4001 poetry run cloakcli ping  # if orchestrator is on same machine

# Or use grpcurl
grpcurl -plaintext 203.0.113.10:4001 list
```

---

## Troubleshooting

### "Failed to locate address in distributed DHT"

**Cause:** The site is not published or the alpha node is offline.

```bash
# Fix: re-host and re-publish
CLOAK_GRPC_PORT=4001 poetry run cloakcli host-static my_site 8080
CLOAK_GRPC_PORT=4001 poetry run cloakcli dht-publish $ADDR
```

### "Bootstrap authority X reachable (same identity or full bucket)"

**Cause:** The bootstrapping node returned the same identity as the joining node, meaning they share a key file. This is not an error if they are the same node; otherwise:

```bash
# Delete the shared data directory and restart
rm -rf data_node-beta/
cargo run -- --port 4002 --id node-beta --bootstrap 127.0.0.1:4001
```

### Node shows 0 relay peers after startup

**Cause:** Bootstrap peer was unreachable at startup time.

```bash
# Check alpha is running
CLOAK_GRPC_PORT=4001 poetry run cloakcli ping

# Check the --bootstrap address resolves
ping 127.0.0.1
# or
ping ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev

# If using Codespaces URL, ensure port is set to Public visibility in the Ports panel
```

### "Could not determine gRPC port for 'hostname'; falling back to port 443"

**Cause:** The URL doesn't have an explicit port and the hostname doesn't have a port embedded in the subdomain.

```bash
# Fix: add an explicit port
--bootstrap "https://myhost.example.com:4001/"
# or use plain format
--bootstrap "myhost.example.com:4001"
```

### Gossip not reaching all nodes

**Cause:** Nodes are not all connected to each other. Node C only bootstrapped to A, so C doesn't appear in B's routing table.

```bash
# Fix: bootstrap to multiple peers
cargo run -- --port 4003 --id node-c \
  --bootstrap 127.0.0.1:4001 \
  --bootstrap 127.0.0.1:4002
```

Or use a `FindNode` walk — after bootstrapping to A, node-C will discover B through A's routing table response. Allow ~5 seconds for propagation after startup.

### Peers file not loading

```bash
# Validate JSON
python3 -c "import json; json.load(open('peers.json'))" && echo "Valid"

# Test normalization
poetry run cloakcli peers-gen "127.0.0.1:4001" --output /tmp/test.json && cat /tmp/test.json
```

### Node crashes immediately with "address already in use"

```bash
# Find what's using the port
ss -tlnp | grep 4001

# Kill the old process if it's a stale cloakmesh
pkill cloakmesh

# Or change the port
cargo run -- --port 4005 --id node-new --bootstrap 127.0.0.1:4001
```

---

## Bootstrap Architecture Reference

### Address normalization pipeline

```
Raw input (CLI --bootstrap or peers.json)
    │
    ▼
normalize_peer_addr(raw: &str)
    │
    ├── starts with http:// or https://?
    │       YES ── url::Url::parse()
    │               ├── has explicit port? → host:port  ✓
    │               └── no explicit port?
    │                       ├── Codespaces pattern? → extract port from subdomain  ✓
    │                       └── fallback → host:443 (HTTPS) or host:80 (HTTP)
    │
    └── no scheme?
            ├── contains ':'? → host:port as-is  ✓
            └── no ':'? → Error: not a valid address
    │
    ▼
Normalized address (host:port)
    │
    ▼
Deduplicate against existing config.bootstrap_peers
    │
    ▼
Added to bootstrap_peers Vec<String>
```

### Bootstrap sequence (Rust)

```rust
// core/src/routing/dht.rs: DhtNode::bootstrap()
for authority_addr in &self.authorities {
    // Phase 1: Ping
    let pong = client.keep_alive(Ping { nonce: 42, sent_at }).await?;
    let peer_id = DhtKey::from_hex(&pong.node_id)?;

    // Add authority to routing table
    self.routing.write().await.add_peer(PeerInfo {
        id: peer_id,
        address: authority_addr.clone(),
        last_seen: Instant::now(),
        reputation: 100,
        flags: Default::default(),
    });

    // Phase 2: FindNode for self
    let resp = client.find_node(FindNodeRequest {
        target_id: hex::encode(self.local_key.0)
    }).await?;

    // Phase 3: Add discovered peers
    for contact in resp.nodes {
        let id = DhtKey::from_hex(&contact.node_id)?;
        self.routing.write().await.add_peer(PeerInfo {
            id,
            address: contact.address,
            last_seen: Instant::now(),
            reputation: 50,
            flags: Default::default(),
        });
    }
}
```

### FindNode response (node.rs)

The `FindNode` handler **always includes self** in the response — this is what allows newly bootstrapped nodes to be discovered by later joiners:

```rust
// core/src/node.rs: find_node handler
let mut closest = routing.find_closest(&target_key, 20);

// Self-advertisement: always include own contact info
closest.push(PeerInfo {
    id: DhtKey(self.identity_pubkey),
    address: format!("127.0.0.1:{}", self.listen_port),
    reputation: 100,
    ..Default::default()
});

// Sort by XOR distance and deduplicate
closest.sort_by(|a, b| a.id.distance(&target_key).cmp(&b.id.distance(&target_key)));
closest.dedup_by(|a, b| a.id.0 == b.id.0);
```

### DHT maintenance (60-second cycle)

After bootstrap, the DHT runs maintenance every 60 seconds:
- Evicts expired storage entries
- (Roadmap Phase 3) Probes stale routing table entries with `KeepAlive`
- (Roadmap Phase 3) Refreshes empty k-buckets with `FindNode` queries

This keeps the routing table fresh even as nodes join and leave.
