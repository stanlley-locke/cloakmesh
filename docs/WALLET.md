# ATK Token System — Wallet Reference

The Anti-Tracking Kernel (ATK) token is CloakMesh's native incentive layer, rewarding nodes for relaying traffic and providing storage capacity.

---

## What Are ATK Tokens?

ATK tokens serve as the economic backbone of the CloakMesh network:

- **Proof-of-Relay:** Nodes earn ATK by forwarding onion circuit traffic
- **Storage Incentive:** Nodes earn ATK by hosting DHT shards (`--mine-atk` flag)
- **Access Payment:** Future hidden services can require ATK payment for access
- **Governance:** ATK holders vote on protocol upgrades (Phase 6)

---

## Architecture

```
wallet transfer 10.0 atk1_recipient
         ↓
Transaction protobuf built
         ↓
BroadcastTx RPC → node gRPC server
         ↓
  Gossip(transaction) → all peers
         ↓
All nodes: queue tx in mempool
         ↓
  [Phase 4] Validate UTXO set
         ↓
  [Phase 4] Add to block / sled ledger
```

---

## Proto Definitions

```proto
// proto/v1/cloakmesh.proto

message UTXO {
    string txid    = 1;   // transaction ID
    uint32 vout    = 2;   // output index
    uint64 amount  = 3;   // value in satoshi-equivalent units
    string address = 4;   // ATK address
}

message TransactionInput {
    string txid      = 1;   // spending UTXO
    uint32 vout      = 2;
    bytes  signature = 3;   // Ed25519 signature of spending tx
    bytes  pubkey    = 4;   // sender's public key
}

message TransactionOutput {
    uint64 amount  = 1;
    string address = 2;   // recipient ATK address
}

message Transaction {
    string                    id      = 1;   // SHA-256 of tx content
    repeated TransactionInput  inputs  = 2;
    repeated TransactionOutput outputs = 3;
    uint64                    timestamp = 4;
}

service CloakMeshNode {
    rpc BroadcastTx (Transaction) returns (TransferAck);
}
```

---

## ATK Address Format

ATK addresses are derived from the node's Ed25519 public key using the same derivation as `.cloak` addresses:

```
atk_address = base32( 0x01 || pubkey[32] || SHA256(SHA256(0x01||pubkey))[:4] ) + "_atk"
```

In the current implementation, addresses are prefixed with `atk1_` for readability. The full cryptographic derivation is performed in `core/src/crypto/mod.rs: derive_address()` and mirrored in `orchestrator/src/cloakcli/cloak_protocol.py`.

---

## BroadcastTx Handler

```rust
// core/src/node.rs
async fn broadcast_tx(&self, request: Request<Transaction>) -> Result<Response<TransferAck>, Status> {
    let tx = request.into_inner();
    tracing::info!("Received BroadcastTx for txid: {}", tx.id);
    // Phase 4: validate UTXO set, check signatures, add to mempool
    // Phase 4: gossip to peers after validation
    Ok(Response::new(TransferAck {
        success: true,
        message: format!("Transaction {} queued", tx.id),
    }))
}
```

The Python `wallet transfer` command currently builds a mock transaction (no real UTXO inputs/outputs) and broadcasts it. The gossip propagation and mempool queuing are real; UTXO validation is roadmapped.

---

## Mining with `--mine-atk`

When the `--mine-atk` flag is passed to the core binary, the storage backend switches from volatile RAM to a persistent `sled` embedded database:

```bash
cargo run -- --port 4003 --id node-miner --mine-atk
```

**What this enables:**
1. Persistent storage of DHT shards across restarts
2. Tracking of relay events for reward calculation
3. Transaction ledger persistence

**Storage location:** `data_node-miner/sled/`

**Future mining mechanism (Phase 4):**
- Relay nodes accumulate `relay_credits` per forwarded byte
- Every N credits = 1 ATK mined
- Credits are signed by circuit endpoints and submitted as `ProofOfRelay` transactions

---

## CLI Usage

### Check balance

```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli wallet balance
```
```
ATK Balance: 100.0 ATK
Derived from your Ed25519 identity key
```

> **Note:** Balance is currently mocked at 100.0 ATK. Phase 4 will query the actual UTXO set filtered by your address.

### Transfer ATK

```bash
CLOAK_GRPC_PORT=4001 poetry run cloakcli wallet transfer 15.5 atk1_recipient_address_xyz
```
```
Success! Sent 15.5 ATK to atk1_recipient_address_xyz
Transaction ID: mock-tx-1234
```

**Node Alpha logs:**
```
Received BroadcastTx for txid: mock-tx-1234
```

**Node Beta + Miner logs (gossip propagation):**
```
[202] GossipMessage ID: <hash> from <sender>
[202] Gossip: queuing tx mock-tx-1234 in mempool
[202] Fanning out gossip <hash> to N peers
```

### Watch transaction propagate across all nodes

```bash
# In separate terminals, watch each node's log
# Then transfer from node-alpha:
CLOAK_GRPC_PORT=4001 poetry run cloakcli wallet transfer 1.0 atk1_test_recv

# All three node terminals should show the gossip within ~1 second
```

---

## Transaction Lifecycle (Full — Phase 4)

```
1. Build Transaction
   - Locate UTXO inputs for sender's address
   - Construct outputs: [recipient_amount, change_back_to_sender]
   - Compute tx_id = SHA256(inputs || outputs || timestamp)
   - Sign with Ed25519: signature = sign(tx_id, identity_key)

2. BroadcastTx RPC
   - Send to any connected node
   - Node validates: UTXO inputs exist, signatures valid, no double-spend
   - Node adds to mempool
   - Node gossips to all peers

3. Block Formation (Phase 4)
   - Mining node batches mempool transactions
   - Computes block hash
   - Appends to sled ledger
   - Broadcasts block via Gossip

4. UTXO Update
   - Consuming inputs: remove from UTXO set
   - Creating outputs: add new UTXOs
   - Update balances
```

---

## Current vs Roadmap

| Feature | Status | Notes |
|---------|--------|-------|
| `BroadcastTx` gRPC | ✅ Done | Real RPC, mock tx content |
| Gossip propagation of tx | ✅ Done | Real fan-out to all peers |
| Balance display | ⚠️ Mocked | Hardcoded 100.0 ATK |
| UTXO input construction | 📅 Phase 4 | Real signature + input spending |
| UTXO validation | 📅 Phase 4 | Double-spend prevention |
| Block formation | 📅 Phase 4 | Sled ledger with sled mining node |
| Proof-of-Relay | 📅 Phase 4 | Credit accumulation per relayed byte |
| ATK governance | 📅 Phase 6 | Voting on protocol upgrades |
