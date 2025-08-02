# Two Animals - Game Server

A Rust-based game server for managing NPC interactions and game state.

## Prerequisites

1. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   Follow the prompts and restart your terminal after installation.

2. **Install cargo-watch** (for auto-reloading during development)
   ```bash
   cargo install cargo-watch
   ```

3. **Install just** (command runner)
   ```bash
   cargo install just
   ```

## Running the Project

1. **Clone and navigate to the project**
   ```bash
   cd Two_Animals
   ```

2. **Run the server**

   **Option A: Development mode with auto-reload**
   ```bash
   just dev
   ```
   This will watch for file changes and automatically restart the server.

   **Option B: Run once**
   ```bash
   just run
   ```

   **Option C: Run directly with cargo**
   ```bash
   cd server
   cargo run
   ```

The server will start on `http://localhost:3000`

## Available Endpoints

- `GET /health` - Health check
- `GET /state` - Get current game state  
- `POST /turn/collect` - Collect NPC intents
- `POST /turn/resolve` - Resolve intents with GM
- `POST /turn/execute` - Execute a full turn (collect + resolve + memory updates)

### Execute Endpoint Options

The `/turn/execute` endpoint accepts a JSON body with the following optional parameters:

- `repeat`: Number of turns to execute (default: 1)
- `endless`: Run turns continuously until stopped (default: false)
- `delay_ms`: Delay between turns in milliseconds (default: 1000)

#### Examples:

**Single turn (default):**
```bash
curl -X POST http://localhost:3000/turn/execute
```

**Multiple turns:**
```bash
curl -X POST http://localhost:3000/turn/execute \
  -H "Content-Type: application/json" \
  -d '{"repeat": 10}'
```

**Multiple turns with custom delay:**
```bash
curl -X POST http://localhost:3000/turn/execute \
  -H "Content-Type: application/json" \
  -d '{"repeat": 5, "delay_ms": 2000}'
```

**Endless mode (runs continuously):**
```bash
curl -X POST http://localhost:3000/turn/execute \
  -H "Content-Type: application/json" \
  -d '{"endless": true}'
```

**Endless mode with custom delay:**
```bash
curl -X POST http://localhost:3000/turn/execute \
  -H "Content-Type: application/json" \
  -d '{"endless": true, "delay_ms": 3000}'
```

The response includes:
- `turns_executed`: Number of turns completed
- `last_turn_result`: The result of the most recent turn
- `status`: Status message indicating completion or interruption

## Development Commands

```bash
just              # List all available commands
just dev          # Run with auto-reload
just dev-log      # Run with auto-reload and save logs to file
just dev-debug    # Run with debug logging and auto-reload
just dev-debug-log # Run with debug logging, auto-reload, and save logs to file
just run          # Run server once
just run-debug    # Run server once with debug logging
just run-trace    # Run server once with trace logging (very verbose)
just check        # Check code without running
just test         # Run tests
just test-verbose # Run tests with output
just test-llm     # Run LLM integration tests (requires Claude CLI)
```

## Logging

By default, the server runs with minimal logging showing only important game events:
- 🚀 Server startup
- 🎮 Turn execution
- 💭 NPC actions
- 🎭 GM decisions
- 📍 Location changes
- ✨ Core memory formation

### Logging to Files

Save all logs to timestamped files in the `logs/` directory:
```bash
just dev-log        # Regular logging to file + console
just dev-debug-log  # Debug logging to file + console
```

Log files are automatically named with timestamps (e.g., `logs/dev_20250802_151230.log`) and excluded from git.

### Debug Logging

For more detailed debugging information:
```bash
just dev-debug    # Auto-reload with debug logs (console only)
just run-debug    # Run once with debug logs
just run-trace    # Run with trace logs (very verbose)
```

Or set the environment variable directly:
```bash
RUST_LOG=debug cargo run
```