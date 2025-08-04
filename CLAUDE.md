# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Two Animals is a Rust-based game server that manages NPC interactions using LLM agents. The system orchestrates turn-based gameplay where NPCs make decisions, interact through contracts, and maintain memories.

**Important**: The server requires LLM provider configuration. Run `./bin/setup-llm.sh` on first use to configure either Claude CLI or Ollama.

### LLM Provider Setup

```bash
./bin/setup-llm.sh         # Interactive setup for LLM provider
```

- **Claude**: Requires Claude CLI installed (see https://docs.anthropic.com/en/docs/claude-cli)
- **Ollama**: Installs as system service
  - Start: `sudo systemctl start ollama`
  - Stop: `sudo systemctl stop ollama`
  - Status: `sudo systemctl status ollama`
  - Pull models: `ollama pull llama3.2:latest`

## Common Commands

### Build and Run

```bash
# Development with auto-reload
just dev                    # Run with auto-reload
just dev-log               # Run with auto-reload and save logs to file
just dev-debug             # Run with debug logging
just dev-debug-log         # Run with debug logging and save to file

# Run once
just run                   # Run server once
just run-debug            # Run with debug logging
just run-trace            # Run with trace logging (very verbose)

# Build and check
just check                 # Check code without running
cd server && cargo build   # Build the project
cd server && cargo fmt     # Format code
```

### Testing

```bash
just test                  # Run all tests
just test-verbose         # Run tests with output
just test-llm             # Run LLM integration tests (requires Claude CLI)
cd server && cargo test [test_name] -- --exact  # Run a specific test
```

### Server Endpoints

The server runs on `http://localhost:3000` with these endpoints:

- `GET /health` - Health check
- `GET /state` - Get current game state
- `POST /turn/collect` - Collect NPC intents
- `POST /turn/resolve` - Resolve intents with GM
- `POST /turn/execute` - Execute full turn with options:
  - `repeat`: Number of turns (default: 1)
  - `endless`: Run continuously (default: false)
  - `delay_ms`: Delay between turns (default: 1000)

## Architecture

### Core Modules (`server/src/`)

- **`game/`** - Game state management, contracts, turn execution
  - `state.rs`: GameStateManager for NPC locations/activities
  - `contracts.rs`: Contract lifecycle management
  - `turn.rs`: Turn orchestration logic
- **`llm/`** - LLM integration
  - `client.rs`: Claude CLI interface
  - `ollama.rs`: Ollama HTTP API client
  - `parser.rs`: Response parsing utilities
- **`npcs/`** - NPC behavior and memory
  - `intent.rs`: Intent collection from NPCs
  - `memory.rs`: Memory system structures
  - `memory_update.rs`: Memory update logic
- **`gm/`** - Game Master logic
  - `resolution.rs`: Intent resolution and reality arbitration
- **`prompts/`** - Prompt construction
  - `builder.rs`: Context-aware prompt building
  - `loader.rs`: Template loading from disk
- **`types.rs`** - Core data structures
- **`lib.rs`** - Router and API handlers

### Data Flow

1. **Turn Execution** (`game/turn.rs`):
   - Process active contracts
   - Collect intents from free NPCs
   - GM resolves all intents
   - Update memories for completed contracts
   - Apply state changes

2. **Contract System**:
   - Contracts lock NPCs into interactions
   - Transcript maintained in `data/contracts/`
   - NPCs freed when contracts end
   - Memory updates triggered on completion

3. **Memory Hierarchy**:
   - Immediate context (current situation)
   - Short-term memory (recent events)
   - Long-term memory (significant events)
   - Motivations (drives and goals)
   - Relationships (feelings about other NPCs)

### Key Patterns

- All LLM calls go through `LlmClient` trait
- Game state accessed via `GameStateManager` (Arc<Mutex>)
- Prompts built dynamically from templates and context
- JSON parsing with fallback strategies in `parser.rs`
- Async operations with Tokio throughout

### File Organization

- `/data/npcs/{name}/` - NPC personalities and memories
- `/data/prompts/` - Prompt templates
- `/data/contracts/` - Active contract transcripts
- `/logs/` - Development logs (git-ignored)
- `/docs/ideas.md` - Game design ideas and future features to implement

