# List available commands
default:
    @just --list

# Run dev server with auto-reload
dev:
    cd server && cargo watch -c -x 'run --bin server'

# Run dev server with debug logging
dev-debug:
    cd server && RUST_LOG=debug cargo watch -c -x 'run --bin server'

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