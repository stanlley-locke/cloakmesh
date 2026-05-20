# SDK Instructions (TypeScript)

## Overview
The `sdk` provides a high-level TypeScript interface for developers to build applications on CloakMesh.

## Architecture
- `src/index.ts`: Main entry point and public API.
- `src/client/`: gRPC and WebSocket communication logic.
- `src/cloak/`: High-level `.cloak` session and address handling.
- `src/crypto/`: WASM-backed cryptographic operations.
- `src/types/`: TypeScript definitions and Zod schemas.

## Coding Standards
- **Strict Typing:** Avoid `any`. Use `unknown` or specific interfaces.
- **Async/Await:** Use async/await for all asynchronous operations.
- **Error Handling:** Always wrap gRPC calls in try/catch and map to `CloakError` types.
- **Functional Style:** Prefer immutable data structures and functional patterns (map, filter, reduce).

## Testing
- **Framework:** `jest`.
- **Unit Tests:** Place in `tests/unit/`.
- **Integration Tests:** Place in `tests/integration/` (requires a running core node).

## WASM Integration
The SDK relies on the `wasm` crate for heavy cryptographic operations. Ensure the WASM module is loaded before performing crypto tasks.
