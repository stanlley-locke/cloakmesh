# Core Engine Instructions (Rust)

## Overview
The `core` crate is the primary engine of the CloakMesh network. It is written in Rust for performance, safety, and reliability.

## Architecture
- `src/main.rs`: Application entry point and runtime bootstrap.
- `src/lib.rs`: Library exports for other crates (e.g., `wasm`).
- `src/crypto/`: Cryptographic primitives and session management.
- `src/network/`: Transport layers (QUIC, TCP) and connection pooling.
- `src/routing/`: DHT, GossipSub, and Onion routing logic.
- `src/cloak_protocol/`: Implementation of the `.cloak` state machine.

## Coding Standards
- **Safety:** Minimize `unsafe` blocks. Every `unsafe` block must have a `// SAFETY:` comment justifying its use.
- **Performance:** Use zero-copy deserialization where possible (e.g., `serde` with `Borrow`).
- **Concurrency:** Prefer `tokio` for async tasks. Avoid blocking the executor with long-running sync tasks; use `spawn_blocking` if necessary.
- **Traits:** Use traits to define interfaces for pluggable components (e.g., `Transport`, `RoutingTable`).

## Testing
- **Unit Tests:** Place in the same file as the code using `#[cfg(test)]`.
- **Integration Tests:** Place in `tests/`. Use `tokio::test` for async tests.
- **Mocks:** Use `mockall` for trait-based mocking in unit tests.

## Documentation
- Use `cargo doc --open` to view internal documentation.
- All public modules and functions must have documentation comments.
