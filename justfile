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
    mkdir -p ../logs
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
    mkdir -p ../logs
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

# Run LLM tests with Claude (requires Claude CLI)
test-claude:
    cd server && TEST_LLM_PROVIDER=claude cargo test --test llm_integration_tests -- --ignored --nocapture

# Run LLM tests with Ollama (requires Ollama running)
test-ollama:
    cd server && TEST_LLM_PROVIDER=ollama cargo test --test llm_integration_tests -- --ignored --nocapture

# Run Ollama-specific tests
test-ollama-specific:
    cd server && cargo test --test ollama_integration_tests -- --ignored --nocapture

# Run Ollama tests with a specific model
test-ollama-model model:
    cd server && TEST_LLM_MODEL={{model}} cargo test --test ollama_integration_tests -- --ignored --nocapture

# Game state management commands
# Clean all game-generated data (contracts and memories)
clean-game-state:
    @echo "⚠️  This will delete all contracts and NPC memories!"
    @echo "Press Ctrl+C to cancel, or Enter to continue..."
    @read
    rm -f data/contracts/*.json
    rm -f data/npcs/*/memories.json
    @echo "✅ Game state cleaned"

# Clean only contracts (keep memories)
clean-contracts:
    @echo "Cleaning contract files..."
    rm -f data/contracts/*.json
    @echo "✅ Contracts cleaned"

# Clean only memories (keep contracts)
clean-memories:
    @echo "Cleaning NPC memory files..."
    rm -f data/npcs/*/memories.json
    @echo "✅ Memories cleaned"

# Reset memories to initial state
reset-memories:
    @echo "Resetting NPC memories to initial state..."
    @cp data/npcs/bear/initial_memories.json data/npcs/bear/memories.json 2>/dev/null || echo "No initial memories for bear"
    @cp data/npcs/wolf/initial_memories.json data/npcs/wolf/memories.json 2>/dev/null || echo "No initial memories for wolf"
    @echo "✅ Memories reset to initial state"

# Initialize game for first time (clean state + initial memories)
init-game:
    @echo "Initializing fresh game state..."
    @just clean-game-state
    @just reset-memories
    @echo "✅ Game initialized with fresh state"

# Backup game state
backup-game-state:
    #!/usr/bin/env bash
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    BACKUP_DIR="backups/game_state_${TIMESTAMP}"
    mkdir -p "${BACKUP_DIR}/contracts"
    mkdir -p "${BACKUP_DIR}/npcs"
    cp -r data/contracts/*.json "${BACKUP_DIR}/contracts/" 2>/dev/null || true
    cp -r data/npcs "${BACKUP_DIR}/" 2>/dev/null || true
    echo "✅ Game state backed up to ${BACKUP_DIR}"

# Restore game state from backup
restore-game-state backup_dir:
    @echo "Restoring from {{backup_dir}}..."
    cp -r {{backup_dir}}/contracts/*.json data/contracts/ 2>/dev/null || true
    cp -r {{backup_dir}}/npcs/*/memories.json data/npcs/ 2>/dev/null || true
    @echo "✅ Game state restored"