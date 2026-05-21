set shell := ["bash", "-c"]
export PATH := env_var_or_default("HOME", "") + "/.cargo/bin:" + env_var_or_default("HOME", "") + "/.local/bin:" + env_var("PATH")

# Bootstrap the project (install all dependencies)
bootstrap: install-deps

# Install all dependencies
install-deps:
    cd core && cargo fetch
    cd orchestrator && poetry install
    cd sdk && npm install

# Regenerate protobuf bindings for all languages
proto-gen:
    mkdir -p {{invocation_directory()}}/orchestrator/src/proto {{invocation_directory()}}/sdk/src/proto
    poetry run -C {{invocation_directory()}}/orchestrator python -m grpc_tools.protoc \
        -I {{invocation_directory()}}/proto/v1 \
        --python_out={{invocation_directory()}}/orchestrator/src/proto \
        --grpc_python_out={{invocation_directory()}}/orchestrator/src/proto \
        {{invocation_directory()}}/proto/v1/*.proto
    # Fix imports in generated python files (relative imports)
    find {{invocation_directory()}}/orchestrator/src/proto -name "*.py" -exec sed -i 's/^import \(.*_pb2\)/from . import \1/g' {} +
    # TypeScript bindings (using proto-loader-gen-types from @grpc/proto-loader)
    {{invocation_directory()}}/sdk/node_modules/.bin/proto-loader-gen-types --longs=String --enums=String --defaults --oneofs --grpcLib=@grpc/grpc-js --outDir={{invocation_directory()}}/sdk/src/proto/ {{invocation_directory()}}/proto/v1/*.proto
    echo "Rust bindings generated via build.rs on cargo build"

# Build all components
build-all: build-core build-wasm

build-core:
    cd core && cargo build --release

build-wasm:
    cd wasm && wasm-pack build --target web --out-dir web/pkg
    cd wasm && cargo build
    cd orchestrator && poetry build
    cd sdk && npm run build

# Run all tests
test:
    cd core && cargo test
    cd orchestrator && poetry run pytest
    cd sdk && npm test

# Lint all components
lint:
    cd core && cargo clippy -- -D warnings
    cd orchestrator && poetry run ruff check src/ && poetry run mypy src/
    cd sdk && npm run lint

# Run a single core node
run-node port="4001" id="node-alpha":
    cd core && cargo run --release -- --port {{port}} --id {{id}}

# Build WASM package
package-wasm:
    cd wasm && wasm-pack build --target web --out-dir web/pkg

# Spin up local 3-node testnet via Docker Compose
testnet-up:
    docker compose -f deploy/docker-compose.yml up -d

testnet-down:
    docker compose -f deploy/docker-compose.yml down

# Full clean
clean:
    cd core && cargo clean
    cd sdk && rm -rf dist node_modules
    cd orchestrator && rm -rf dist .venv
