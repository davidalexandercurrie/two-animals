# Two Animals - Technical Architecture

## System Components

### 1. Rust Server (`/server`)

- Manages game state and NPC metadata
- Coordinates turn-based actions
- Handles contract lifecycle
- Calls Claude CLI for NPC decisions
- Routes:
  - `GET /state` - Current game state
  - `POST /turn/collect` - Collect intents from free NPCs
  - `POST /turn/resolve` - Send intents to GM for resolution
  - `POST /turn/execute` - Execute complete turn (contracts + intents + GM)

### 2. NPC Agents (`/npcs/{name}`)

Each NPC folder contains:

- `CLAUDE.md` - Personality and response format
- `memories.json` - Personal memories and relationships with structure:
  ```json
  {
    "immediate_context": "Current situation",
    "short_term": ["Recent events"],
    "long_term": ["Core memories"],
    "motivations": [
      {"type": "basic", "drive": "hungry"},
      {"type": "goal", "drive": "find the missing ranger"}
    ],
    "relationships": {"wolf": "wary"}
  }
  ```

### 3. GM Agent (`/gm`)

- `CLAUDE.md` - Instructions for arbitrating reality
- Resolves simultaneous intents
- Manages contract creation
- Writes shared truth files

### 4. Shared Data (`/shared`)

- `/shared/contracts/` - Contract transcripts
- `/shared/events/` - Recent events log
- `/shared/truth/` - GM's authoritative record

## Data Flow

### Turn Execution

```
1. Server triggers turn (every 15 seconds)
2. Process Active Contracts:
   - For each NPC in contract: get response to contract context
   - Update contract transcripts
   - Check if contracts are ending
3. Process Free NPCs:
   - For each NPC without contract: "What do you do next?"
   - Receive intent JSON
4. GM Resolution:
   - Send all intents (from contracts and free NPCs) to GM
   - GM returns reality, state changes, and next prompts
   - For contract NPCs: prompts include rich context of what just happened
5. Contract Cleanup:
   - For ended contracts: trigger memory summarization
   - Each participant saves personal memory
   - Free NPCs from ended contracts
6. Update game state with:
   - New NPC locations/activities
   - New/updated contracts
   - Clear processed intents
```

### Intent Format

```json
{
  "npc": "bear",
  "thought": "I'm getting hungry",
  "action": "Head to the river to fish",
  "dialogue": "See you later, Wolf"
}
```

NPCs express what they want to do naturally. The GM interprets these intents and determines actual state changes. NPCs in contracts also return full intents with thoughts and actions, not just dialogue.

### GM Response Format

```json
{
  "reality": "Description of what actually happened",
  "state_changes": [{
    "npc": "bear",
    "location": "DeepForest",
    "activity": "approaching Wolf cautiously"
  }],
  "contracts": [{
    "type": "conversation",
    "participants": ["bear", "wolf"],
    "state": "pending",
    "id": "conv_12345"
  }],
  "next_prompts": {
    "bear": "Wolf's hackles are raised and he growled 'This is my territory'. He's blocking the path to the river. How do you respond?",
    "wolf": "Bear stopped and is looking at you uncertainly. They seem to want to pass. What do you do?"
  },
  "contract_updates": [{
    "id": "conv_12345",
    "action": "add_to_transcript",
    "entry": {
      "actor": "wolf",
      "action": "blocked path with raised hackles",
      "dialogue": "This is my territory",
      "target_reaction": "Bear stopped, looking uncertain"
    }
  }]
}
```

## Contract Management

### Contract Lifecycle

```
1. GM creates contract based on intents
2. Contract is immediately active (NPCs are in it)
3. NPCs interact within contract context
4. Either NPC can leave/end with their action
5. Cleanup: memories saved, contract archived
```

No state machine needed - contracts exist or they don't.

### Contract Data Structure

```rust
struct Contract {
    id: String,
    participants: Vec<String>,
    transcript_file: String,  // Path to interaction transcript
}

struct Npc {
    name: String,
    location: Location,
    activity: String,
    folder_path: String,
    active_contract: Option<String>,  // Contract ID if locked
}

// No ContractType or ContractState needed
// Contracts exist (active) or don't exist
// The GM decides what goes in the contract based on intents
```

## Memory System

### Hierarchical Memory

1. **Immediate Context** - Current situation, active contracts (overwritten each turn)
2. **Short-term Memory** - Recent events, last 5-10 turns (gradually fades)
3. **Long-term Memory** - Significant events, core experiences
4. **Motivations** - Drives that influence behavior:
   - Basic needs (hungry, tired)
   - Goals (find something, learn something)
   - Emotional (upset, curious, vengeful)
5. **Relationships** - Feelings about other NPCs, updated after interactions

### Memory Updates

During turn:
1. Update immediate context with current situation
2. Add significant events to short-term memory
3. Fade old short-term memories

After contract ends:
1. Each participant reads contract transcript
2. Claude prompted: "What do you remember from this interaction?"
3. Personal summary saved to memories
4. Relationships updated
5. New motivations may be created

## Claude Integration

### NPC Prompt Construction

```
[Read CLAUDE.md]
[Read current context]
[Read relevant memories]
[Insert specific prompt based on situation]
```

### Safety Considerations

- Server validates all Claude outputs
- Intents must match expected schema
- File writes go through server
- No direct file system access from Claude

## Future Considerations

### Scalability

- Move to event queue system
- Separate turn processing from web server
- Database for persistent state

### Features

- More contract types
- Environmental events
- Player integration
- Observation system (who can see what)

