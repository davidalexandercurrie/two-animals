# List available commands
default:
    @just --list

# Run dev server with auto-reload
dev:
    cd server && cargo watch -c -x 'run --bin server'

# Run dev server with auto-reload and log to file
dev-log:
    #!/usr/bin/env bash
    cd server
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    echo "Logging to logs/dev_${TIMESTAMP}.log"
    cargo watch -c -x 'run --bin server' 2>&1 | tee "../logs/dev_${TIMESTAMP}.log"

# Run dev server with debug logging
dev-debug:
    cd server && RUST_LOG=debug cargo watch -c -x 'run --bin server'

# Run dev server with debug logging and log to file
dev-debug-log:
    #!/usr/bin/env bash
    cd server
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    echo "Debug logging to logs/dev_debug_${TIMESTAMP}.log"
    RUST_LOG=debug cargo watch -c -x 'run --bin server' 2>&1 | tee "../logs/dev_debug_${TIMESTAMP}.log"

# Run server once
run:
    cd server && cargo run --bin server

# Run server with debug logging
run-debug:
    cd server && RUST_LOG=debug cargo run --bin server

# Run server with trace logging (very verbose)
run-trace:
    cd server && RUST_LOG=trace cargo run --bin server

# Check code without running
check:
    cd server && cargo check

# Run tests
test:
    cd server && cargo test

# Run tests with output
test-verbose:
    cd server && cargo test -- --nocapture

# Run integration tests that call real LLM API (may cost money!)
test-llm:
    cd server && cargo test --test llm_integration_tests -- --ignored --nocapture