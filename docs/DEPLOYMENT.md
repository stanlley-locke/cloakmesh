# CloakMesh Deployment Guide

Instructions for deploying CloakMesh nodes locally, on a VPS, or as a systemd service.

---

## Local Multi-Node Development Setup

The most common setup during development is three nodes on localhost. See [TESTING_GUIDE.md](../TESTING_GUIDE.md) for the full testing walkthrough.

### Quick Start (3 nodes)

**Terminal 1 — Bootstrap node:**
```bash
cd /workspaces/cloakmesh/core
cargo run --release -- --port 4001 --id node-alpha
```

**Terminal 2 — Relay node:**
```bash
cargo run --release -- --port 4002 --id node-beta --bootstrap 127.0.0.1:4001
```

**Terminal 3 — Mining node:**
```bash
cargo run --release -- --port 4003 --id node-miner --mine-atk \
  --bootstrap 127.0.0.1:4001 \
  --bootstrap 127.0.0.1:4002
```

**Terminal 4 — Orchestrator:**
```bash
cd /workspaces/cloakmesh/orchestrator
CLOAK_GRPC_PORT=4001 poetry run cloakcli interactive
```

---

## Background Node Management (Python CLI)

Start and stop nodes without keeping terminals open:

```bash
cd /workspaces/cloakmesh/orchestrator

# Start node-alpha
poetry run cloakcli node-start --port 4001 --id node-alpha

# Start node-beta (bootstraps to alpha)
poetry run cloakcli node-start --port 4002 --id node-beta --bootstrap 127.0.0.1:4001

# Start node-miner
poetry run cloakcli node-start --port 4003 --id node-miner --bootstrap 127.0.0.1:4001

# Stop nodes
poetry run cloakcli node-stop --port 4001
poetry run cloakcli node-stop --port 4002
poetry run cloakcli node-stop --port 4003
```

PID files are stored in `/tmp/cloakmesh/node-<port>.pid`.

> **Note:** `node-start` runs the binary from the current working directory. The Rust binary must be built first: `cargo build --release`

---

## Configuration File

Create `core/configs/default.toml` to customize node behavior:

```toml
node_id      = "my-node"
listen_port  = 4001
bootstrap_peers = [
    "known-peer.example.com:4001",
    "another-peer.example.com:4001"
]
data_dir = "data"   # auto-becomes data_<node_id>/

[crypto]
identity_key_path = "data/identity.pem"
pq_hybrid_enabled = true

[circuit]
default_hops = 3

[traffic]
cell_size_bytes = 256
padding_enabled = true

[log_level]
# "trace" | "debug" | "info" | "warn" | "error"
```

Without this file, the node uses hardcoded defaults (port 4001, 3-hop circuits, etc.).

---

## Production VPS Deployment

### Build Release Binary

```bash
cd /workspaces/cloakmesh
cargo build --release -p cloakmesh-core

# Binary location:
./target/release/cloakmesh
```

### Deploy to VPS

```bash
scp ./target/release/cloakmesh user@vps-ip:/opt/cloakmesh/
scp -r core/configs/ user@vps-ip:/opt/cloakmesh/
```

### systemd Service Unit

`/etc/systemd/system/cloakmesh.service`:
```ini
[Unit]
Description=CloakMesh Privacy Network Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=cloakmesh
WorkingDirectory=/opt/cloakmesh
ExecStart=/opt/cloakmesh/cloakmesh \
    --port 4001 \
    --id my-public-node \
    --mine-atk \
    --bootstrap bootstrap1.cloakmesh.net:4001 \
    --bootstrap bootstrap2.cloakmesh.net:4001
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start
systemctl daemon-reload
systemctl enable cloakmesh
systemctl start cloakmesh

# View logs
journalctl -u cloakmesh -f
```

---

## Firewall Configuration

```bash
# Allow gRPC port (inbound from mesh peers)
ufw allow 4001/tcp comment "CloakMesh gRPC"

# SOCKS5 is localhost-only (do NOT open externally)
# 127.0.0.1:9050 is already localhost-bound
```

---

## Data Directory Structure

```
data_node-alpha/
├── identity.pem        # Ed25519 private key (KEEP SECRET)
└── sled/               # Persistent DHT storage (--mine-atk only)
    ├── db               # sled database file
    └── ...
```

**Back up `identity.pem`** — losing it means losing your `.cloak` address permanently.

---

## Port Reference

| Port | Protocol | Direction | Purpose |
|------|----------|-----------|---------|
| 4001 | gRPC (TCP) | Inbound | Core gRPC server (peers connect here) |
| 9050 | SOCKS5 (TCP) | Loopback only | .cloak browser gateway |
| 4002 | gRPC (TCP) | Inbound | Second node (if running) |
| 9051 | SOCKS5 (TCP) | Loopback only | Second node gateway |

---

## Health Checks

```bash
# Quick ping (Python CLI)
CLOAK_GRPC_PORT=4001 poetry run cloakcli ping

# Check node status
CLOAK_GRPC_PORT=4001 poetry run cloakcli status

# Check known peers
CLOAK_GRPC_PORT=4001 poetry run cloakcli relays

# Curl through SOCKS5
curl --socks5-hostname 127.0.0.1:9050 http://example-site.cloak/
```

---

## Log Analysis

Nodes log JSON structured events with the `tracing` crate. Key log codes:

| Code | Meaning |
|------|---------|
| `[200]` | Operational — node started, server listening |
| `[201]` | Bootstrap — DHT peer discovery events |
| `[202]` | Gossip — message propagation events |
| `[9000]` | Config warning — non-fatal configuration issue |

```bash
# Filter for DHT bootstrap events
journalctl -u cloakmesh | grep '"201"'

# Watch gossip propagation live
journalctl -u cloakmesh -f | grep '"202"'

# Full JSON log parsing with jq
journalctl -u cloakmesh | jq 'select(.fields.message | contains("bootstrap"))'
```
