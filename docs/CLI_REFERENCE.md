# CloakMesh CLI Reference

Complete reference for all `cloakcli` commands. Set `CLOAK_GRPC_PORT=<port>` to select which node to connect to (default: 4001).

```bash
cd /workspaces/cloakmesh/orchestrator
export CLOAK_GRPC_PORT=4001   # target node-alpha
```

---

## Node Operations

### `ping`
Check if the node is alive and measure round-trip latency.

```bash
poetry run cloakcli ping
```
```
PONG node_id=0707886184709d2c... latency=1.4ms
```

| Exit | Meaning |
|------|---------|
| `PONG` printed | Node is live |
| `Ping failed` | Node offline or wrong port |

---

### `status`
Display node metrics in a table.

```bash
poetry run cloakcli status
```
```
         Node Status
Metric          │ Value
────────────────┼─────────────────────────────
Address         │ aedqpcdb...cloak
Active Circuits │ 1
Bytes Relayed   │ 0
Reputation      │ 1.0
```

---

### `dash`
Launch the live full-screen terminal dashboard. Refreshes every second.

```bash
poetry run cloakcli dash
# Press Ctrl+C to exit
```

Panels:
- **Core Metrics** — uptime, circuits, DHT entries, bytes relayed, latency, reputation
- **Active Circuits** — top 5 circuits with ID, hops, status
- **Core Event Stream** — last 20 lines of `core.log` (JSON structured logs)

---

### `stream-metrics`
Stream live telemetry pushed every 2 seconds from the node. Press Ctrl+C to stop.

```bash
poetry run cloakcli stream-metrics
```
```
uptime=42s circuits=1 dht_entries=3 relayed=0.0KB latency=0ms rep=1.000
uptime=44s circuits=1 dht_entries=3 relayed=0.2KB latency=0ms rep=1.000
```

---

### `circuits`
List all active onion circuits.

```bash
poetry run cloakcli circuits
```
```
╭─ Active Onion Circuits ──────────────────────────────────────╮
│ Circuit ID       │ Hops │ Status  │ Latency                  │
│ 91fcb75e-fae4... │ 3    │ READY   │ 0ms                      │
╰──────────────────────────────────────────────────────────────╯
```

---

### `relays`
List known DHT relay peers from the routing table.

```bash
poetry run cloakcli relays
```
```
╭─ Known DHT Relay Peers ──────────────────────────────────────╮
│ Node ID          │ Address          │ Reputation │ Latency   │
│ 0707886184709d2c │ 127.0.0.1:4001  │ 100        │ 0ms       │
╰──────────────────────────────────────────────────────────────╯
```

---

### `address`
Show this node's `.cloak` address derived from its Ed25519 identity.

```bash
poetry run cloakcli address
```
```
╭─ Your Node Identity ───────────────────────────────────────────╮
│ aedqpcdbqryj2lbj66mcup6relejktkzcp2kzt5fdz72tgvlpiemavqil5uq.cloak │
╰────────────────────────────────────────────────────────────────╯
```

---

## Web Hosting & Browsing

### `site-init <dir_name>`
Scaffold a new dark-themed static `.cloak` website.

```bash
poetry run cloakcli site-init my_site
```
```
SUCCESS: Created beautiful static site in my_site/index.html
```

Creates `<dir_name>/index.html` with glassmorphism dark theme.

---

### `host-static <dir_name> [port]`
Start a Python HTTP server on `port`, bridge it to the mesh under your node's `.cloak` address, and auto-publish the DHT descriptor with your intro point.

```bash
poetry run cloakcli host-static my_site 8080
```

| Argument | Default | Description |
|----------|---------|-------------|
| `dir_name` | required | Directory to serve |
| `port` | 8080 | Local HTTP port |

```
SUCCESS: Started static server on port 8080 and bridged to <addr>.cloak
Auto-publishing to DHT...
SUCCESS: Descriptor for <addr>.cloak published successfully
Your site is LIVE! Browse it with: cloakcli browse <addr>.cloak
```

**What happens:**
1. `python3 -m http.server <port> -d <dir>` starts in background thread
2. `HostSite` gRPC call registers address→port in the SOCKS5 bridge
3. A `CloakDescriptor` with `IntroductionPoint{address: "127.0.0.1:<grpc_port>"}` is stored in DHT and replicated to peers
4. `PublishDescriptor` gRPC also sends it from Python side

---

### `host <address> <port>`
Bridge an existing local service on `port` to a specific `.cloak` address. Use this when you already have a service running.

```bash
poetry run cloakcli host aedqpcdb...cloak 8080
```

| Argument | Description |
|----------|-------------|
| `address` | `.cloak` address to bind to |
| `port` | Local TCP port of the service |

---

### `browse <address>`
Fetch the index page of a `.cloak` site by routing through the SOCKS5 proxy.

```bash
ADDR=aedqpcdb...cloak
CLOAK_GRPC_PORT=4002 poetry run cloakcli browse $ADDR
```

Uses `CLOAK_GRPC_PORT + 5049` as the SOCKS5 port. Sends `HTTP GET /` and prints the raw response.

```
Attempting to fetch index from <addr>.cloak via Core SOCKS5 Proxy (127.0.0.1:9051)...

--- RESPONSE ---
HTTP/1.0 200 OK
...
<!DOCTYPE html>...
----------------
```

**Tip:** Use a different node's `CLOAK_GRPC_PORT` than the one hosting the site to test cross-node routing.

---

## DHT Operations

### `dht-publish <address>`
Publish a `CloakDescriptor` for `address` to the Kademlia DHT, including an `IntroductionPoint` with the current node's gRPC address.

```bash
poetry run cloakcli dht-publish aedqpcdb...cloak
```
```
Publishing descriptor for aedqpcdb...cloak...
SUCCESS: Descriptor for aedqpcdb...cloak published successfully
```

---

### `dht-fetch <address>`
Retrieve the `CloakDescriptor` for `address` from the DHT.

```bash
poetry run cloakcli dht-fetch aedqpcdb...cloak
```
```
Fetching descriptor for aedqpcdb...cloak...
SUCCESS: Descriptor found
  Address: aedqpcdb...cloak
  Pubkey:  0707886184...
  Version: 1
  IntroPoint: 127.0.0.1:4001
```

---

## Messaging

### `chat-send <message> <target>`
Send an encrypted chat message. The message streams through the node's `ChatStream` gRPC and is echoed back from the relay.

```bash
poetry run cloakcli chat-send "Hello from the darknet!" node-alpha
```
```
Sending chat message from 'node-alpha'...
Relay response from 'CloakMesh Relay': Hello from the darknet!
```

| Argument | Description |
|----------|-------------|
| `message` | Text to send |
| `target` | Destination node identifier |

---

### `listen-chat`
Open a blocking chat listener. Prints any incoming messages. Press Ctrl+C to stop.

```bash
poetry run cloakcli listen-chat
```
```
Listening for incoming chat messages... Press Ctrl+C to stop.
CloakMesh Relay: Hello!
```

---

### `view-chat-history`
Display the recent chat message history table.

```bash
poetry run cloakcli view-chat-history
```
```
      Mesh Message History
Timestamp        │ Sender   │ Message
2026-05-21 14:22 │ Stanlley │ Hello through the onion!
2026-05-21 14:30 │ Relay-1  │ Awaiting handshake...
```

---

### `file-send <path> <target>`
Send a file to `target` over the mesh via gRPC `FileTransfer` (client streaming). Files are split into 64KB chunks.

```bash
poetry run cloakcli file-send secret.txt node-alpha
```
```
Sharing file 'secret.txt' with node-alpha...
SUCCESS: File received: secret.txt (35 bytes)
```

| Argument | Description |
|----------|-------------|
| `path` | Path to the local file |
| `target` | Destination address or node ID |

---

### `file-recv`
Wait for incoming file transfers.

```bash
poetry run cloakcli file-recv
```
```
Waiting for incoming files...
Incoming file: secret.txt (35 bytes)
Saved to: ./downloads/secret.txt
```

---

### `list-files`
List all files received from the mesh.

```bash
poetry run cloakcli list-files
```
```
      Received Mesh Files
Filename   │ Size  │ Origin         │ Status
secret.txt │ 35B   │ ahqw6...cloak  │ DECRYPTED
schema.pdf │ 1.2MB │ node-7...cloak │ READY
```

---

## Decentralized Storage

### `storage upload <file_path>`
Hash a file, broadcast the manifest via gossip, and store the shard in the DHT.

```bash
poetry run cloakcli storage upload secret.txt
```
```
File Size: 55 bytes
SHA-256: a3f8d2c1b4e5f6a7...
Broadcasting File Manifest to network via Gossip...
Distributing data shards into DHT...
Upload Complete! File is decentralized.
To download, use: cloakcli storage download a3f8d2c1b4e5f6a7...
```

---

### `storage download <file_hash> [--output/-o <path>]`
Retrieve a file shard from the DHT and write it to disk.

```bash
poetry run cloakcli storage download a3f8d2c1b4e5f6a7... --output recovered.txt
# or short form:
poetry run cloakcli storage download a3f8d2c1b4e5f6a7... -o recovered.txt
```

```
Locating file a3f8d2c1b4... in the Mesh DHT...
Success! File reconstructed → recovered.txt (55 bytes)
```

| Argument | Default | Description |
|----------|---------|-------------|
| `file_hash` | required | SHA-256 hex hash from upload |
| `--output/-o` | `downloaded_file.bin` | Output file path |

---

## ATK Wallet

### `wallet balance`
Display your ATK token balance.

```bash
poetry run cloakcli wallet balance
```
```
ATK Balance: 100.0 ATK
Derived from your Ed25519 identity key
```

---

### `wallet transfer <amount> <address>`
Broadcast an ATK token transfer. The transaction propagates through gossip to all nodes.

```bash
poetry run cloakcli wallet transfer 15.5 atk1_recipient_address_xyz
```
```
Success! Sent 15.5 ATK to atk1_recipient_address_xyz
Transaction ID: mock-tx-1234
```

| Argument | Description |
|----------|-------------|
| `amount` | Float amount of ATK to send |
| `address` | Recipient ATK address |

---

## Capability Tokens

### `auth-issue <address> [scope] [ttl]`
Issue a capability token granting access to a `.cloak` address.

```bash
poetry run cloakcli auth-issue aedqpcdb...cloak read 3600
poetry run cloakcli auth-issue aedqpcdb...cloak write 7200
```

| Argument | Default | Description |
|----------|---------|-------------|
| `address` | required | Target `.cloak` address |
| `scope` | `read` | Access scope: `read`, `write`, `admin` |
| `ttl` | `3600` | Token lifetime in seconds |

```json
{"token_id":"a3f8d2c1b4e5f6a7","cloak_address":"aedqpcdb...cloak","scope":"read","expires_at":1748254800,"signature":""}
```

---

### `auth-verify '<token_json>'`
Parse and verify a capability token (expiry, fields).

```bash
TOKEN=$(poetry run cloakcli auth-issue $ADDR read 3600 2>/dev/null | tail -1)
poetry run cloakcli auth-verify "$TOKEN"
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

---

## Node Lifecycle

### `node-start`
Start a CloakMesh node as a background process.

```bash
poetry run cloakcli node-start --port 4004 --id node-delta --bootstrap 127.0.0.1:4001
```

| Option | Default | Description |
|--------|---------|-------------|
| `--port/-p` | 4001 | gRPC listen port |
| `--id` | — | Node identifier |
| `--bootstrap` | — | Bootstrap peer (host:port) |
| `--mine-atk` | false | Enable ATK mining |

```
Node started on port 4004 (pid 12345)
Node started on gRPC port 4004 | SOCKS5 on 9053
```

---

### `node-stop`
Stop a background node by port.

```bash
poetry run cloakcli node-stop --port 4004
```
```
Node 12345 stopped
```

---

## Interactive Console

### `interactive`
Launch the interactive REPL console for all commands.

```bash
poetry run cloakcli interactive
```
```
╭─────────────────────────────────────────────────────────────╮
│  Welcome to the CloakMesh Interactive Console               │
│  Type 'help' to see commands.                               │
╰─────────────────────────────────────────────────────────────╯
cloak> help
cloak> status
cloak> browse
```

Type `exit`, `quit`, or `q` to exit.

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CLOAK_GRPC_PORT` | `4001` | gRPC port of the target node |

**SOCKS5 port:** always `CLOAK_GRPC_PORT + 5049`

| Node Port | SOCKS5 Port |
|-----------|-------------|
| 4001 | 9050 |
| 4002 | 9051 |
| 4003 | 9052 |
| 4004 | 9053 |

---

## Using cURL Through SOCKS5

```bash
# Browse via node-beta's proxy
curl --socks5-hostname 127.0.0.1:9051 http://aedqpcdb...cloak/

# With verbose output
curl -v --socks5-hostname 127.0.0.1:9051 http://aedqpcdb...cloak/

# Download a file
curl --socks5-hostname 127.0.0.1:9051 http://aedqpcdb...cloak/document.pdf -o doc.pdf
```
