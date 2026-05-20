# CloakMesh Architecture Overview

## System Layers

CloakMesh is designed with a layered, polyglot architecture to ensure high performance, developer flexibility, and broad platform support.

### 1. Core Engine (Rust)
The "heart" of the node. It handles all low-level networking, cryptography, and protocol state machines.
- **Transport:** QUIC, TCP, and libp2p bridge.
- **Crypto:** Ed25519, X25519, Noise handshakes.
- **Routing:** Kademlia DHT, Onion routing, Circuit management.
- **Storage:** KV store for descriptors, session cache.

### 2. Orchestrator (Python)
The management layer. Used for CLI interaction, node orchestration, and system-wide analytics.
- **CLI:** `cloakcli` for node control and health checks.
- **Automation:** Deployment scripts and testnet management.
- **Analytics:** Traffic analysis and reputation scoring simulations.

### 3. SDK & Client (TypeScript)
The interface for developers. Provides high-level APIs for building applications on top of CloakMesh.
- **Session Management:** High-level `.cloak` connection handling.
- **WASM Integration:** Offloads heavy crypto to compiled Rust modules.
- **Polyfill:** Browser-compatible networking adapters.

## Data Lifecycle

1.  **Message Origination:** An application uses the SDK to send a message.
2.  **Encryption:** The message is encrypted end-to-end using session keys.
3.  **Layering:** The Core engine wraps the message in multiple layers of encryption (Onion).
4.  **Transmission:** The cell is sent through a pre-established circuit of relays.
5.  **Rendezvous:** The message passes through a Rendezvous Point to the destination service.
6.  **Decryption:** The destination service peels off the layers and decrypts the payload.

## Component Map

```text
[ SDK (TS) ] <--- gRPC/WS ---> [ Core (Rust) ] <--- P2P (QUIC) ---> [ Relays/IPs/RPs ]
      ^                             ^
      |                             |
      +--- [ Orchestrator (Py) ] ---+
```

## Security Boundaries

- **Local:** Communication between the SDK/Orchestrator and the Core is via gRPC (encrypted/authenticated locally).
- **Network:** All inter-node communication is encrypted using Noise handshakes.
- **End-to-End:** Service payloads are encrypted with keys known only to the client and service.
