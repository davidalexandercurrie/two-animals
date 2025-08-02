# Two Animals - Refactoring Plan

## Overview

This document outlines the plan to refactor the Two Animals codebase for better maintainability, LLM portability, and feature extensibility. The key principles:

1. **Modular code structure** - Split monolithic main.rs into focused modules
2. **LLM-agnostic design** - All context provided via prompts, no LLM-specific file conventions
3. **Clear data separation** - Markdown for static content, JSON for runtime mutable data
4. **Composable prompts** - Build prompts from reusable components

## File Structure

### Current Structure (To Be Refactored)
```
Two_Animals/
├── server/
│   └── src/
│       ├── main.rs (431 lines - too monolithic)
│       └── types.rs
├── npcs/
│   ├── bear/
│   │   ├── CLAUDE.md
│   │   └── memories.json
│   └── wolf/
│       ├── CLAUDE.md
│       └── memories.json
└── gm/
    └── CLAUDE.md
```

### New Structure
```
Two_Animals/
├── server/
│   └── src/
│       ├── main.rs              # ~50 lines - just setup and routing
│       ├── lib.rs               # Public API and re-exports
│       ├── types.rs             # Keep existing types
│       ├── config.rs            # Configuration management
│       ├── game/
│       │   ├── mod.rs           # Game engine coordination
│       │   ├── state.rs         # GameState management
│       │   ├── turn.rs          # Turn execution logic
│       │   └── contracts.rs     # Contract lifecycle
│       ├── npcs/
│       │   ├── mod.rs           # NPC management
│       │   ├── registry.rs      # NPC data loading
│       │   ├── intent.rs        # Intent collection
│       │   └── memory.rs        # Memory system implementation
│       ├── prompts/
│       │   ├── mod.rs           # Prompt system
│       │   ├── loader.rs        # Load markdown templates
│       │   └── builder.rs       # Compose complete prompts
│       ├── claude/
│       │   ├── mod.rs           # LLM interface abstraction
│       │   ├── client.rs        # Command execution
│       │   └── parser.rs        # Response parsing
│       └── gm/
│           ├── mod.rs           # GM logic
│           └── resolution.rs    # Intent resolution
└── data/
    ├── npcs/
    │   ├── bear/
    │   │   ├── personality.md   # Static personality (Markdown)
    │   │   └── memories.json    # Mutable memories (JSON)
    │   └── wolf/
    │       ├── personality.md
    │       └── memories.json
    ├── prompts/
    │   ├── core/
    │   │   └── npc_base.md      # Core NPC instructions
    │   └── gm/
    │       └── gm_base.md       # GM arbitration rules
    └── contracts/               # Runtime contract storage
        └── *.json
```

## Data Philosophy

### Markdown Files (Static During Runtime)
- **Purpose**: Define static aspects of the system
- **Content**: Personalities, base instructions, response formats
- **Location**: `data/npcs/*/personality.md`, `data/prompts/`
- **Loaded**: At startup, cached in memory
- **Modified**: Only during development/testing

### JSON Files (Mutable During Runtime)
- **Purpose**: Store dynamic game state
- **Content**: Memories, relationships, contracts, game state
- **Location**: `data/npcs/*/memories.json`, `data/contracts/`
- **Loaded**: On-demand, persisted after updates
- **Modified**: During gameplay by the system

## Module Responsibilities

### `game/` - Core Game Logic
- Manages game state and turn execution
- Coordinates between NPCs, contracts, and GM
- Handles persistence and state recovery

### `npcs/` - NPC Management
- Loads NPC data from filesystem
- Implements memory system
- Collects intents from NPCs
- No LLM-specific logic

### `prompts/` - Prompt Construction
- Loads markdown templates
- Composes prompts from multiple sources
- Injects dynamic context (memories, contracts, game state)
- Returns complete prompt strings

### `claude/` - LLM Interface
- Abstracts LLM command execution
- Handles response parsing and error recovery
- Easy to swap for other LLMs (ollama, OpenAI, etc.)

### `gm/` - Game Master Logic
- Processes intents into reality
- Manages simultaneous actions
- Creates and updates contracts

## Implementation Phases

### Phase 1: Core Refactoring (Week 1)
1. Create module structure
2. Extract game state management to `game/state.rs`
3. Move intent collection to `npcs/intent.rs`
4. Extract LLM calls to `claude/client.rs`
5. Reduce main.rs to routing only

### Phase 2: Prompt System (Week 1-2)
1. Migrate CLAUDE.md content to new structure
2. Implement `prompts/loader.rs` for markdown loading
3. Create `prompts/builder.rs` for composition
4. Update all LLM calls to use new prompt system
5. Remove old npcs/ and gm/ folders

### Phase 3: Memory System (Week 2)
1. Implement full memory system in `npcs/memory.rs`
2. Create memory update pipeline
3. Add memory fading logic
4. Integrate with prompt building

### Phase 4: Configuration & Polish (Week 3)
1. Add `config.rs` for game configuration
2. Improve error handling throughout
3. Add comprehensive tests
4. Update documentation

## Example Code Snippets

### Prompt Building
```rust
// prompts/builder.rs
pub fn build_npc_intent_prompt(
    npc_name: &str,
    loader: &PromptLoader,
    game_state: &GameState,
    memories: &MemorySystem,
) -> Result<String> {
    let sections = vec![
        loader.load_npc_base()?,           // Core instructions
        loader.load_personality(npc_name)?, // Character personality
        format_current_state(npc_name, game_state),
        format_memories(memories),
        format_active_contract(npc_name, game_state),
        "What do you do next?".to_string(),
    ];
    
    Ok(sections.join("\n\n---\n\n"))
}
```

### LLM Abstraction
```rust
// claude/client.rs
pub trait LlmClient {
    async fn query(&self, prompt: String, working_dir: &Path) -> Result<String>;
}

pub struct ClaudeClient;
impl LlmClient for ClaudeClient {
    async fn query(&self, prompt: String, working_dir: &Path) -> Result<String> {
        // Current claude CLI implementation
    }
}

// Future: OllamaClient, OpenAIClient, etc.
```

### NPC Data Loading
```rust
// npcs/registry.rs
pub struct NpcRegistry {
    npcs: HashMap<String, NpcData>,
}

impl NpcRegistry {
    pub fn load_from_directory(data_dir: &Path) -> Result<Self> {
        let mut npcs = HashMap::new();
        
        for entry in fs::read_dir(data_dir.join("npcs"))? {
            let name = entry?.file_name().to_string_lossy().to_string();
            let npc_data = NpcData::load(&name, data_dir)?;
            npcs.insert(name, npc_data);
        }
        
        Ok(Self { npcs })
    }
}
```

## Testing Strategy

### Unit Tests
- Prompt building logic
- Memory system operations
- Contract state transitions
- GM resolution logic

### Integration Tests
- Full turn execution
- Memory persistence
- Multi-NPC interactions
- Contract lifecycle

### LLM Mocking
```rust
#[cfg(test)]
pub struct MockLlmClient {
    responses: HashMap<String, String>,
}

impl LlmClient for MockLlmClient {
    async fn query(&self, prompt: String, _: &Path) -> Result<String> {
        // Return predefined responses for testing
    }
}
```

## Migration Checklist

- [ ] Create new directory structure
- [ ] Set up module skeleton
- [ ] Migrate game state logic
- [ ] Migrate intent collection
- [ ] Create prompt system
- [ ] Migrate CLAUDE.md content
- [ ] Implement memory system
- [ ] Add configuration
- [ ] Write tests
- [ ] Update documentation
- [ ] Remove old structure

## Success Criteria

1. **Modularity**: No file larger than 200 lines
2. **Testability**: 80%+ code coverage
3. **LLM Portability**: Can swap LLM with single trait implementation
4. **Performance**: No regression in turn execution time
5. **Maintainability**: Clear separation of concerns

## Risks and Mitigation

### Risk: Over-engineering for 2-NPC scope
**Mitigation**: Keep abstractions simple, focus on immediate needs

### Risk: Breaking existing functionality
**Mitigation**: Incremental refactoring, maintain tests throughout

### Risk: Complex prompt debugging
**Mitigation**: Add prompt logging, visual prompt builder tools

## Future Benefits

This refactoring enables:
- Easy addition of new NPCs
- Support for multiple LLM providers
- Rich memory and relationship systems
- Better testing and debugging
- Community contributions (clearer structure)