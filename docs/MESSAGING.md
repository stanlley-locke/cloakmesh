# CloakMesh Messaging System

Complete reference for all peer-to-peer messaging features: encrypted chat, streaming file transfer, and gossip propagation.

---

## Architecture Overview

CloakMesh messaging is built on gRPC streaming RPCs defined in `proto/v1/cloakmesh.proto`. All communication passes through the Rust core's gRPC server, which routes through onion circuits.

```
Sender (Python CLI)              Core gRPC Server (Rust)         Recipient
     │                                    │                          │
     │  ChatStream (bidirectional)        │                          │
     │ ─────────────────────────────────→ │  relay + echo            │
     │ ←─────────────────────────────────  │                          │
     │                                    │                          │
     │  FileTransfer (client streaming)   │                          │
     │ chunk1 ────────────────────────→  │                          │
     │ chunk2 ────────────────────────→  │                          │
     │ ...is_last=true ───────────────→  │  TransferAck             │
     │ ←─────────────────────────────────  │                          │
     │                                    │                          │
     │  Gossip (unary fan-out)            │                          │
     │  GossipMessage ─────────────────→ │ → peer1                  │
     │                                    │ → peer2                  │
     │                                    │ → peer3                  │
```

---

## Chat: `ChatStream` RPC

### Proto Definition

```proto
// proto/v1/cloakmesh.proto
service CloakMeshNode {
    rpc ChatStream(stream ChatMessage) returns (stream ChatMessage);
}

message ChatMessage {
    string sender   = 1;
    string text     = 2;
    Timestamp sent_at = 3;
}
```

### How It Works

`ChatStream` is a **bidirectional streaming** RPC. The client sends `ChatMessage` objects and the server echoes them back with `sender = "CloakMesh Relay"`. In Phase 4, this relay will be replaced with actual encrypted delivery to the specified recipient.

**Rust handler (`node.rs: chat_stream`):**
```rust
async fn chat_stream(&self, request: Request<Streaming<ChatMessage>>) -> ... {
    let (tx, rx) = mpsc::channel(32);
    let mut stream = request.into_inner();
    tokio::spawn(async move {
        while let Some(msg) = stream.next().await {
            let msg = msg.unwrap();
            // Echo back with relay attribution
            tx.send(Ok(ChatMessage {
                sender: "CloakMesh Relay".to_string(),
                text: msg.text,
                sent_at: msg.sent_at,
            })).await.unwrap();
        }
    });
    Ok(Response::new(ReceiverStream::new(rx)))
}
```

**Python client (`communication.py: send_chat_message`):**
```python
def send_chat_message(text: str, sender: str):
    client = CloakGrpcClient()
    def message_generator():
        ts = Timestamp()
        ts.FromSeconds(int(time.time()))
        yield ChatMessage(sender=sender, text=text, sent_at=ts)

    responses = client.chat_stream(message_generator())
    for response in responses:
        print(f"Relay response from '{response.sender}': {response.text}")
```

### CLI Usage

#### Send a chat message
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli chat-send "Hello from the darknet!" node-alpha
```
```
Sending chat message from 'node-alpha'...
Relay response from 'CloakMesh Relay': Hello from the darknet!
```

#### Listen for incoming messages
Open in Terminal A:
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli listen-chat
```
```
Listening for incoming chat messages... Press Ctrl+C to stop.
```

Send from Terminal B:
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli chat-send "Yo!" sender-beta
```
Terminal A shows:
```
CloakMesh Relay: Yo!
```

#### View chat history
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

## File Transfer: `FileTransfer` RPC

### Proto Definition

```proto
service CloakMeshNode {
    rpc FileTransfer(stream FileChunk) returns (TransferAck);
}

message FileChunk {
    string file_id      = 1;   // unique ID for this transfer session
    string filename     = 2;   // original filename
    bytes  data         = 3;   // chunk payload (max 64KB)
    uint64 chunk_index  = 4;   // 0-based chunk number
    bool   is_last      = 5;   // true on final chunk
}

message TransferAck {
    bool   success = 1;
    string message = 2;
}
```

### How It Works

`FileTransfer` is a **client-streaming** RPC. The client sends chunks and the server assembles them. Files are read in 64KB windows; `is_last` is set true when the final chunk is sent.

**Python client (`communication.py: share_file`):**
```python
def share_file(file_path: str, target: str):
    client = CloakGrpcClient()
    file_id = f"file_{int(time.time())}"

    def chunk_generator():
        with open(file_path, "rb") as f:
            chunk_index = 0
            while True:
                data = f.read(64 * 1024)   # 64KB chunks
                is_last = len(data) < (64 * 1024)
                yield FileChunk(
                    file_id=file_id,
                    filename=os.path.basename(file_path),
                    data=data,
                    chunk_index=chunk_index,
                    is_last=is_last or not data
                )
                if is_last or not data:
                    break
                chunk_index += 1

    ack = client.file_transfer(chunk_generator())
    # ack.success, ack.message
```

**Rust handler (`node.rs: file_transfer`):**
```rust
async fn file_transfer(&self, request: Request<Streaming<FileChunk>>) -> ... {
    let mut stream = request.into_inner();
    let mut total_bytes = 0u64;
    let mut filename = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        filename = chunk.filename.clone();
        total_bytes += chunk.data.len() as u64;
    }
    Ok(Response::new(TransferAck {
        success: true,
        message: format!("File received: {} ({} bytes)", filename, total_bytes),
    }))
}
```

### CLI Usage

#### Send a file
```bash
echo "Confidential document" > secret.txt
CLOAK_GRPC_PORT=4001 poetry run cloakcli file-send secret.txt node-alpha
```
```
Sharing file 'secret.txt' with node-alpha...
SUCCESS: File received: secret.txt (22 bytes)
```

#### Send a large file (multiple chunks)
```bash
dd if=/dev/urandom bs=1024 count=200 of=large_file.bin 2>/dev/null
CLOAK_GRPC_PORT=4001 poetry run cloakcli file-send large_file.bin node-alpha
```
The file is split into 64KB chunks: `chunk_index: 0, 1, 2, ...`

#### Listen for incoming files
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli file-recv
```
```
Waiting for incoming files...
Incoming file: secret.txt (22 bytes)
Saved to: ./downloads/secret.txt
```

#### List received files
```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli list-files
```
```
      Received Mesh Files
Filename   │ Size  │ Origin         │ Status
secret.txt │ 22B   │ ahqw6...cloak  │ DECRYPTED
schema.pdf │ 1.2MB │ node-7...cloak │ READY
```

---

## Gossip Protocol: `Gossip` RPC

### Proto Definition

```proto
service CloakMeshNode {
    rpc Gossip(GossipMessage) returns (TransferAck);
}

message GossipMessage {
    string message_id     = 1;   // SHA-256 hash of content (dedup key)
    string sender_address = 2;   // originating ATK address
    oneof payload {
        Transaction  transaction = 3;
        FileManifest manifest    = 4;
    }
}
```

### How It Works

Gossip is a **unary** RPC that triggers network-wide epidemic broadcast. When a node receives a `GossipMessage`:

1. **Deduplication:** Check `seen_gossip: HashSet<String>`. If `message_id` already seen, drop immediately.
2. **Local processing:** Queue transaction in mempool or note file manifest.
3. **Fan-out:** Re-send `GossipMessage` to all peers in the DHT routing table (spawn one gRPC call per peer, fire-and-forget).

```rust
// node.rs
async fn gossip(&self, request: Request<GossipMessage>) -> ... {
    let msg = request.into_inner();

    // Dedup
    let mut seen = self.seen_gossip.write().await;
    if !seen.insert(msg.message_id.clone()) {
        return Ok(Response::new(TransferAck { success: true, message: "Already seen".into() }));
    }
    drop(seen);

    // Fan-out to all known peers
    let peers = self.dht.list_peers().await;
    for peer in peers {
        let msg_clone = msg.clone();
        tokio::spawn(async move {
            let addr = format!("http://{}", peer.address);
            if let Ok(mut client) = CloakMeshNodeClient::connect(addr).await {
                let _ = client.gossip(Request::new(msg_clone)).await;
            }
        });
    }
}
```

### Gossip Triggers

| Event | Payload |
|-------|---------|
| `storage upload` | `FileManifest{file_hash, size, shard_hashes, owner_address}` |
| `wallet transfer` | `Transaction{id, inputs, outputs, timestamp}` |

### Convergence

In a connected mesh of N nodes each knowing K peers:
- Each gossip message reaches all nodes in O(log N) rounds
- Deduplication prevents storms: each node processes each message exactly once
- No coordinator needed — fully decentralized

### Verifying Gossip in Logs

After a `storage upload` or `wallet transfer`, check all node terminals:

```
# Originating node (node-alpha):
[202] GossipMessage ID: a3f8d2c1... from atk1_demo
[202] Gossip: saw file manifest hash=a3f8d2c1...
[202] Fanning out gossip a3f8d2c1... to 2 peers

# Relay nodes (node-beta, node-miner):
[202] GossipMessage ID: a3f8d2c1... from atk1_demo
[202] Gossip: saw file manifest hash=a3f8d2c1...
[202] Fanning out gossip a3f8d2c1... to 1 peers
```

---

## Capability-Gated Messaging (Roadmap)

The `Introduce` RPC supports `capability_token` in its payload:

```proto
message Introduce1 {
    string intro_point_id   = 1;
    bytes  encrypted_payload = 2;
    bytes  capability_token  = 3;   // JSON token from auth-issue
}
```

In Phase 4, the intro point will verify the token via `CapabilityService.Verify` before forwarding the introduction, enabling access-controlled hidden services.

---

## Phase Roadmap for Messaging

| Phase | Feature | Status |
|-------|---------|--------|
| Phase 2 | gRPC ChatStream relay echo | ✅ Done |
| Phase 2 | gRPC FileTransfer (client streaming) | ✅ Done |
| Phase 3 | Gossip fan-out with dedup | ✅ Done |
| Phase 4 | Double Ratchet E2E encryption | 📅 Planned |
| Phase 4 | Persistent encrypted mailbox | 📅 Planned |
| Phase 4 | Recipient address routing in ChatStream | 📅 Planned |
| Phase 5 | Sealed sender (Signal-style) | 📅 Planned |
| Phase 5 | Group messaging via gossip channels | 📅 Planned |
