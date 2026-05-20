# CloakMesh Project Instructions

Welcome to the CloakMesh project. This file provides foundational mandates, architectural overview, and development workflows for AI agents and contributors.

## Project Vision
CloakMesh is a decentralized, privacy-first peer-to-peer network designed for metadata-resistant communication. It combines onion-style routing, Kademlia DHT, and a polyglot architecture to provide a secure foundation for private messaging, decentralized AI, and censorship-resistant infrastructure.

## Core Mandates

### 1. Security & Privacy First
- **Zero Metadata Leakage:** All protocol changes must be analyzed for metadata leaks.
- **Cryptographic Integrity:** Use established primitives (Ed25519, X25519, ChaCha20-Poly1305). Prefer the implementations in `core/src/crypto/`.
- **No PII:** Never log or transmit personally identifiable information or unencrypted session metadata.

### 2. Polyglot Consistency
- The project spans **Rust** (Core), **Python** (Orchestrator), and **TypeScript** (SDK).
- **Protobuf is the Source of Truth:** All cross-language communication must use the schemas defined in `proto/v1/`.
- Always run `just proto-gen` after modifying `.proto` files to ensure all layers are in sync.

### 3. Error Handling
- **Rust:** Use `anyhow` for application-level errors and `thiserror` (or similar idiomatic patterns) for library errors.
- **TypeScript:** Use custom error classes defined in `sdk/src/utils/errors.ts`.
- **Python:** Use Pydantic for validation and standard exception hierarchy.

## Project Structure

- `core/`: The heart of the network. High-performance Rust engine for transport, routing, and crypto.
- `orchestrator/`: Python-based CLI and management tools for deployment and testing.
- `sdk/`: TypeScript SDK for building client applications and web interfaces.
- `wasm/`: Rust-to-WASM bindings for running CloakMesh in the browser.
- `proto/v1/`: Protobuf definitions for the `.cloak` protocol.
- `docs/`: Technical specifications and guides.

## Development Workflow

### Task Runner: `Justfile`
Use `just` for common tasks:
- `just build-all`: Compiles all components.
- `just test`: Runs all tests across languages.
- `just proto-gen`: Regenerates gRPC/Protobuf bindings.
- `just lint`: Runs linters for all languages.

### Branching & Commits
- Follow conventional commits (`feat:`, `fix:`, `docs:`, `perf:`, `refactor:`).
- All PRs must pass `just lint` and `just test`.

## Technical Standards

### Rust (Core)
- Edition: 2021.
- Async Runtime: `tokio`.
- Logging: `tracing` with `tracing-subscriber`.
- Documentation: Use `///` for public APIs.

### TypeScript (SDK)
- Target: ES2020+.
- Strict mode enabled.
- Documentation: TSDoc.

### Python (Orchestrator)
- Python 3.10+.
- Dependency Management: `poetry`.
- Type Hinting: Mandatory for all new code.

## Phase 1 Objectives
The current focus is **Protocol Definition & Cross-Language Bindings**.
- Ensure `proto/v1/` schemas are complete and cover Handshake, DHT, and Cloak Service operations.
- Maintain robust codegen pipeline in `Justfile`.
- Implement smoke tests for generated bindings in all three languages.

## Subdirectory Instructions
- [Core Instructions](./core/GEMINI.md)
- [SDK Instructions](./sdk/GEMINI.md)
- [Orchestrator Instructions](./orchestrator/GEMINI.md)
