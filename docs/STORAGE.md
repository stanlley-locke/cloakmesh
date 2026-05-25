# AetherStore — Decentralized File Storage

Technical reference for CloakMesh's decentralized storage layer, which uses the Kademlia DHT as the storage backbone for sharded, gossip-replicated file distribution.

---

## Architecture

```
File Upload Flow:

  User file
    ↓ SHA-256 hash
  FileManifest (metadata)
    ↓ Gossip RPC (broadcast to all peers)
  FileShard (raw data)
    ↓ StoreShard RPC → DHT store_local()
  Stored under key: SHA256("file_hash:shard_index")
  TTL: 24 hours


File Download Flow:

  file_hash
    ↓ RetrieveShard RPC
  DHT get_local(key=SHA256("file_hash:0"))
    ↓
  FileShard{data: raw_bytes}
    ↓ write to output file
```

---

## Proto Definitions

```proto
// proto/v1/cloakmesh.proto

message FileShard {
    string file_hash    = 1;   // SHA-256 of original file
    uint32 shard_index  = 2;   // 0-based shard number
    bool   is_parity    = 3;   // true = erasure parity shard
    bytes  data         = 4;   // raw shard payload
}

message RetrieveShardRequest {
    string file_hash   = 1;
    uint32 shard_index = 2;
}

message FileManifest {
    string file_hash              = 1;   // SHA-256 of complete file
    uint64 size_bytes             = 2;
    uint32 data_shards            = 3;   // current: 1
    uint32 parity_shards          = 4;   // current: 0
    repeated string shard_hashes  = 5;   // SHA-256 per shard
    string owner_address          = 6;   // sender's ATK address
    uint64 timestamp              = 7;
}

message GossipMessage {
    string message_id     = 1;
    string sender_address = 2;
    oneof payload {
        Transaction  transaction = 3;
        FileManifest manifest    = 4;   // ← used for storage
    }
}

service CloakMeshNode {
    rpc StoreShard    (FileShard)             returns (TransferAck);
    rpc RetrieveShard (RetrieveShardRequest) returns (FileShard);
    rpc Gossip        (GossipMessage)        returns (TransferAck);
}
```

---

## DHT Key Derivation

File shards are stored in the Kademlia DHT under a deterministic key:

```rust
// core/src/node.rs: store_shard handler
let key_input = format!("{}:{}", req.file_hash, req.shard_index);
let key_bytes = sha2::Sha256::digest(key_input.as_bytes());
let dht_key = DhtKey(key_bytes.into());
```

So for a file with `file_hash = "a3f8d2..."` and `shard_index = 0`:
```
dht_key = SHA256("a3f8d2...:0")
```

This ensures shards from the same file map to adjacent nodes in the XOR keyspace, enabling efficient retrieval.

---

## StoreShard Handler

```rust
// core/src/node.rs
async fn store_shard(&self, request: Request<FileShard>) -> Result<Response<TransferAck>, Status> {
    let shard = request.into_inner();
    let key_input = format!("{}:{}", shard.file_hash, shard.shard_index);
    let key_bytes = sha2::Sha256::digest(key_input.as_bytes());
    let key = DhtKey(key_bytes.into());

    self.dht.store_local(key, shard.data.to_vec(), Duration::from_secs(86400))  // 24h TTL
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(TransferAck { success: true, message: "Shard stored".into() }))
}
```

---

## RetrieveShard Handler

```rust
async fn retrieve_shard(&self, request: Request<RetrieveShardRequest>) -> Result<Response<FileShard>, Status> {
    let req = request.into_inner();
    let key_input = format!("{}:{}", req.file_hash, req.shard_index);
    let key = DhtKey(sha2::Sha256::digest(key_input.as_bytes()).into());

    match self.dht.get_local(&key).await {
        Ok(Some(data)) => Ok(Response::new(FileShard {
            file_hash: req.file_hash,
            shard_index: req.shard_index,
            is_parity: false,
            data: data.into(),
        })),
        Ok(None) => Err(Status::not_found("Shard not found in local DHT")),
        Err(e) => Err(Status::internal(e.to_string())),
    }
}
```

---

## Storage Backends

### Volatile RAM (Default)

```rust
// core/src/routing/dht.rs
pub struct StorageBackend {
    store: ZeroizeWrapper<HashMap<DhtKey, StoredValue>>,
}
```

- All data lives in RAM
- `ZeroizeWrapper` zeroes memory on drop (prevents forensic recovery)
- Entries expire after TTL (cleaned by `start_maintenance()` every 60s)
- **Data is lost on node restart**

### Persistent Storage (`--mine-atk`)

```rust
// When --mine-atk flag is set:
pub struct StorageBackend {
    store: sled::Db,  // Embedded LSMT key-value store
}
```

- Data survives node restart
- Enables ATK reward tracking across sessions
- Stored in `data_<node_id>/sled/`

---

## Python Upload Flow

`orchestrator/src/cloakcli/storage.py: upload()`:

```python
def upload(file_path: str):
    with open(file_path, "rb") as f:
        data = f.read()

    # 1. Compute identifiers
    file_hash = hashlib.sha256(data).hexdigest()
    shard_hash = file_hash   # single shard = entire file

    # 2. Build FileManifest
    manifest = FileManifest(
        file_hash=file_hash,
        size_bytes=len(data),
        data_shards=1,
        parity_shards=0,
        shard_hashes=[shard_hash],
        owner_address="atk1_demo",
        timestamp=0
    )

    # 3. Broadcast manifest via Gossip (reaches all peers)
    gossip_msg = GossipMessage(
        message_id=file_hash,
        sender_address="atk1_demo",
        manifest=manifest
    )
    client.Gossip(gossip_msg)

    # 4. Store the shard in DHT
    shard = FileShard(
        file_hash=shard_hash,
        shard_index=0,
        is_parity=False,
        data=data
    )
    resp = client.StoreShard(shard)
```

---

## CLI Usage

### Upload a file

```bash
# Create test file
echo "Confidential mesh document" > document.txt

# Upload via node-alpha
CLOAK_GRPC_PORT=4001 poetry run cloakcli storage upload document.txt
```

```
File Size: 27 bytes
SHA-256: a3f8d2c1b4e5f6a7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1
Broadcasting File Manifest to network via Gossip...
Distributing data shards into DHT...
Upload Complete! File is decentralized.
To download, use: cloakcli storage download a3f8d2c1b4e5f6a7...
```

**Node logs (alpha):**
```
[202] GossipMessage ID: a3f8d2c1... from atk1_demo
[202] Gossip: saw file manifest hash=a3f8d2c1...
[202] Fanning out gossip a3f8d2c1... to 2 peers
Stored shard a3f8d2c1...:0 in DHT (27 bytes, TTL=24h)
```

**Node logs (beta and miner) — gossip arrived:**
```
[202] GossipMessage ID: a3f8d2c1... from atk1_demo
[202] Gossip: saw file manifest hash=a3f8d2c1...
[202] Fanning out gossip a3f8d2c1... to 1 peers
```

### Download — same node

```bash
FILE_HASH=a3f8d2c1b4e5f6a7...

CLOAK_GRPC_PORT=4001 poetry run cloakcli storage download $FILE_HASH --output recovered.txt
cat recovered.txt
```

```
Locating file a3f8d2c1b4... in the Mesh DHT...
Success! File reconstructed → recovered.txt (27 bytes)
Confidential mesh document
```

### Download — different node

```bash
# Shard must exist on node-beta for this to work
# (currently only stored on the uploading node; cross-node DHT replication roadmapped)
CLOAK_GRPC_PORT=4002 poetry run cloakcli storage download $FILE_HASH -o from_beta.txt
```

### Upload a binary file

```bash
dd if=/dev/urandom bs=1024 count=16 of=random.bin 2>/dev/null
CLOAK_GRPC_PORT=4001 poetry run cloakcli storage upload random.bin
```

### Batch upload

```bash
for f in *.txt; do
  CLOAK_GRPC_PORT=4001 poetry run cloakcli storage upload "$f"
done
```

---

## Current Implementation: Single-Shard

The current AetherStore implementation uses a single data shard with no parity:

| Parameter | Value |
|-----------|-------|
| `data_shards` | 1 |
| `parity_shards` | 0 |
| Erasure coding | Not implemented (roadmap) |
| Max file size | RAM-limited (volatile) |
| Shard TTL | 24 hours |
| Cross-node replication | Via gossip (manifest only; shard replication planned) |

---

## Erasure Coding Roadmap (Phase 6)

The proto schema already supports the full erasure coding model:

```
FileManifest
├── data_shards: N       (e.g. 8)
├── parity_shards: M     (e.g. 4)
└── shard_hashes: [N+M]  (SHA-256 per shard)
```

Full erasure coding (Reed-Solomon) will:
1. Split file into N data shards
2. Compute M parity shards
3. Distribute all N+M shards to different DHT nodes
4. Allow reconstruction from any N out of N+M shards
5. Provide M-node fault tolerance

Target: `rs-erasure` crate in Rust core.

---

## Error Scenarios

| Error | Cause | Fix |
|-------|-------|-----|
| `Shard not found in local DHT` | File not uploaded to this node | Upload first, or enable cross-node replication |
| `RPC Error: StatusCode.UNAVAILABLE` | Node not running | Start the node, check CLOAK_GRPC_PORT |
| File content corrupted | SHA-256 mismatch | Re-upload; check disk integrity |
| `Upload Failed` from StoreShard | DHT storage full or error | Check node logs for details |
