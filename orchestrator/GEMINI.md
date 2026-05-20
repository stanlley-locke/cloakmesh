# Orchestrator Instructions (Python)

## Overview
The `orchestrator` is a Python-based toolset for managing, testing, and automating CloakMesh nodes.

## Architecture
- `src/cloakcli/main.py`: Typer-based CLI entry point.
- `src/cloakcli/node_manager.py`: Logic for starting and monitoring core processes.
- `src/cloakcli/api/`: gRPC client stubs and wrappers.
- `tests/`: Pytest suite for CLI and protocol conformance.

## Coding Standards
- **Type Hints:** Use PEP 484 type hints for all function signatures.
- **Formatting:** Code must be formatted with `black` and linted with `ruff`.
- **Dependency Management:** Use `poetry`.
- **CLI Design:** Follow the Typer patterns for subcommands and arguments.

## Testing
- **Framework:** `pytest`.
- **Async:** Use `pytest-asyncio` for testing gRPC clients.
- **Mocks:** Use `unittest.mock` for isolating CLI logic from the filesystem or network.

## Integration
The orchestrator communicates with the core node via gRPC. Always ensure the core node is running and the gRPC port matches the configuration.
