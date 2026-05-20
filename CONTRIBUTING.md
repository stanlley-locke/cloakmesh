# Contributing to CloakMesh

First off, thank you for considering contributing to CloakMesh! It's people like you that make the decentralized web a reality.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct (found in `CODE_OF_CONDUCT.md`).

## How Can I Contribute?

### Reporting Bugs
- Use GitHub Issues.
- Describe the bug, steps to reproduce, and expected vs actual behavior.
- For security vulnerabilities, please email **security@cloakmesh.network**. Do NOT open public issues for security findings.

### Suggesting Enhancements
- Open a GitHub Issue with the tag `enhancement`.
- Provide a clear and concise description of the proposed feature.

### Pull Requests
1. Fork the repository.
2. Create a new branch for your feature or fix.
3. Ensure your code follows the project's style and passes all tests:
   ```bash
   just lint
   just test
   ```
4. Submit a Pull Request with a clear description of the changes.

## Development Setup

See the [Getting Started](#getting-started) section in the `README.md` for instructions on setting up your development environment.

## Style Guidelines

### Rust
- Use `cargo fmt` and `cargo clippy`.
- Follow the patterns in `core/GEMINI.md`.

### TypeScript
- Use `npm run lint`.
- Follow the patterns in `sdk/GEMINI.md`.

### Python
- Use `black` and `ruff`.
- Follow the patterns in `orchestrator/GEMINI.md`.

## Protobuf Changes
If you modify any files in `proto/v1/`, you MUST run `just proto-gen` and ensure all generated code is updated.

---
"Privacy isn't a feature. It's a foundation."
