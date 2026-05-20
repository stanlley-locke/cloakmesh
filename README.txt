================================================================================
  CLOAKMESH
  A Decentralized, Privacy-First Peer-to-Peer Network
================================================================================

LICENSE: Apache-2.0
LANGUAGES: Rust (1.70+), Python (3.10+), TypeScript (5.0+)
STATUS: Active Development
DOCS: docs/PROTOCOL.md | security@cloakmesh.network

================================================================================
  OVERVIEW
================================================================================

CloakMesh is a next-generation decentralized network designed to give developers
and users control over data, identity, and communication. Built from the ground
up with privacy, resilience, and modularity in mind, it combines:

  - Onion-style layered routing for metadata-resistant communication
  - Kademlia DHT + GossipSub for robust peer discovery & message propagation
  - Cryptographic self-sovereign identities (Ed25519 + DID-compatible)
  - Polyglot architecture: Rust (core), Python (orchestration/ML), TS (SDK/Web)
  - WASM-ready for in-browser peers & edge deployment
  - Open protocol spec with language-agnostic contract definitions

Whether you're building private messaging, decentralized AI training,
censorship-resistant storage, or community infrastructure, CloakMesh provides
the secure, extensible foundation you need.

================================================================================
  ARCHITECTURE
================================================================================

  [CLIENT LAYER (TS/JS)]
    - Web UI / Light Client / SDK / Browser WASM Peer
          ^
          | gRPC / WebSocket / REST
          v
  [ORCHESTRATOR LAYER (PYTHON)]
    - Node Management / CLI / ML Analytics / Test Harness
          ^
          | gRPC / Protobuf / IPC
          v
  [CORE ENGINE LAYER (RUST)]
    - P2P Transport / DHT / Onion Routing / Crypto / WASM Export

TECH STACK BREAKDOWN:
  Layer             | Language   | Key Dependencies              | Purpose
  ------------------|------------|-------------------------------|--------------------------
  Core Network      | Rust       | tokio, libp2p, ring, tonic    | Transport, crypto, routing
  Orchestration     | Python     | grpcio, typer, asyncio, pandas| CLI, node mgmt, analytics
  Client/SDK        | TypeScript | @grpc/grpc-js, zod, wasm-pack | Dev SDK, web UI, events
  Protocol          | Protobuf+CBOR | protoc, serde, cbor       | Language-agnostic contracts

================================================================================
  SECURITY & PRIVACY MODEL
================================================================================

  IDENTITY       : Ed25519 keypairs -> cloak:<base32_hash>:<service>
  ENCRYPTION     : Noise_XX_25519_ChaChaPoly_BLAKE2s + perfect forward secrecy
  ROUTING        : 3-hop onion circuits (Guard -> Middle -> Exit) w/ ephemeral keys
  METADATA MIN.  : Fixed-size packet padding, traffic blending, optional cover streams
  FWD SECRECY    : Ephemeral session keys rotated per circuit; past sessions safe
  SYBIL RESIST.  : Pluggable: PoW puzzles, stake-backed trust, or social validation

  NOTE: CloakMesh protects against network-level surveillance and single-point
  failures. Application-layer leaks (credentials, behavioral patterns) must be
  mitigated at the integration level.

================================================================================
  PROJECT STRUCTURE
================================================================================

  cloakmesh/
  ├── core/                 # Rust: networking, crypto, routing, consensus
  ├── orchestrator/         # Python: CLI, node mgmt, analytics, testing
  ├── sdk/                  # TypeScript: SDK, web client, event system
  ├── proto/                # Shared .proto & CBOR schema definitions
  ├── wasm/                 # Compiled WASM artifacts & bindgen configs
  ├── deploy/               # Docker, systemd, K8s, CI/CD pipelines
  ├── docs/                 # Protocol spec, RFCs, threat model, guides
  ├── Justfile              # Unified task runner (build, test, run)
  └── README.txt

================================================================================
  GETTING STARTED
================================================================================

PREREQUISITES:
  - Rust >= 1.70 (rustup)
  - Python >= 3.10 (pip + poetry recommended)
  - Node.js >= 18 (npm or pnpm)
  - protoc >= 3.20

QUICK INSTALL:
  git clone https://github.com/your-org/cloakmesh.git
  cd cloakmesh
  curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin
  just bootstrap

RUN A LOCAL NETWORK:
  1. Build all components:
     just build-all

  2. Start 3 bootstrap/core nodes (separate terminals):
     just run-core --port 4001 --id node-alpha
     just run-core --port 4002 --id node-beta
     just run-core --port 4003 --id node-gamma

  3. Connect Python CLI orchestrator:
     cd orchestrator && poetry run cloakcli connect --bootstrap 127.0.0.1:4001

  4. Launch web dashboard / SDK:
     cd sdk && npm run dev
     Open http://localhost:3000

DOCKER COMPOSE (FASTEST START):
  version: "3.9"
  services:
    node-alpha:
      build: ./core
      ports: ["4001:4001"]
      environment: { CLOAK_ID: "alpha", BOOTSTRAP: "" }
    node-beta:
      build: ./core
      ports: ["4002:4002"]
      environment: { CLOAK_ID: "beta", BOOTSTRAP: "node-alpha:4001" }
    orchestrator:
      build: ./orchestrator
      depends_on: [node-alpha]
      command: ["cloakcli", "start", "--bootstrap", "node-alpha:4001"]

  Run: docker compose up -d && docker compose logs -f orchestrator

================================================================================
  DEVELOPMENT WORKFLOW
================================================================================

KEY COMMANDS (Justfile):
  just build-all      : Compile Rust core, install Python deps, build TS SDK
  just test           : Run unit + integration tests across all layers
  just lint           : cargo clippy, ruff, eslint + format checks
  just proto-gen      : Regenerate gRPC/Protobuf bindings for all languages
  just run-e2e        : Spin up 5-node testnet & run protocol conformance
  just package-wasm   : Build browser-ready WASM + JS glue

ADDING A NEW PROTOCOL MODULE:
  1. Define message in proto/module_v1.proto
  2. Run just proto-gen
  3. Implement handler in core/src/protocols/module_v1.rs
  4. Add Python/TS bindings via generated gRPC stubs
  5. Write cross-language tests in sdk/tests/ & orchestrator/tests/

================================================================================
  PROTOCOL SPECIFICATION
================================================================================

The core protocol is defined in docs/PROTOCOL.md and covers:
  - Addressing format & DID compatibility
  - Handshake & key exchange flow
  - Circuit establishment & teardown
  - Message framing & serialization (CBOR + Protobuf)
  - Error codes & retry semantics
  - Governance & upgrade mechanisms

Full spec: docs/PROTOCOL.md
Reference impl: core/src/protocol/

================================================================================
  ROADMAP
================================================================================

  Phase  | Milestone                           | Status
  -------|-------------------------------------|----------
  v0.1   | Core transport + crypto + DHT       | DONE
  v0.2   | Onion routing + gRPC polyglot bridge| IN PROGRESS
  v0.3   | WASM browser peer + TS SDK v1       | PLANNED
  v0.4   | Reputation/ML scoring + Sybil resist| PLANNED
  v1.0   | Mainnet testnet + audit + hardening | PLANNED

================================================================================
  CONTRIBUTING
================================================================================

We welcome contributors! Please follow our guidelines:
  1. Read CONTRIBUTING.md & CODE_OF_CONDUCT.md
  2. Fork, branch, and sign your commits (git commit -S)
  3. Run just lint && just test before opening PRs
  4. Tag PRs: feat, fix, docs, perf, security
  5. Join community channels for design reviews & syncs

SECURITY DISCLOSURES:
  Please report vulnerabilities to security@cloakmesh.network
  (PGP key available). Do not open public issues for security findings.

================================================================================
  LICENSE
================================================================================

Licensed under Apache-2.0 (c) 2026 CloakMesh Contributors
See LICENSE for full terms. Cryptographic components may carry additional
attribution requirements per upstream licenses (ring, libp2p, etc.).

================================================================================
  LINKS & COMMUNITY
================================================================================

  Docs        : docs.cloakmesh.network
  Matrix/Elem : #cloakmesh:matrix.org
  GitHub      : github.com/your-org/cloakmesh
  Social      : @CloakMeshNet (X/Bluesky)
  Testnet     : testnet.cloakmesh.network

================================================================================
  "Privacy isn't a feature. It's a foundation."
  Thank you for building a more open, secure, and user-sovereign internet.
================================================================================