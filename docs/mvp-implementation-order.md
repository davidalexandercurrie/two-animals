# MVP Implementation Order

Building the minimal testable backend for Two Animals.

## What We've Built So Far

### ✅ Basic HTTP Server

- Health check endpoint: `GET /health`
- Game state endpoint: `GET /state`
- Returns current NPC locations and activities

### ✅ Game State Structure

```rust
struct GameState {
    npcs: HashMap<String, Npc>,
}

struct Npc {
    name: String,
    location: Location,
    activity: String,
    folder_path: String,
}
```

### ✅ Bear NPC Setup

- Created `/npcs/bear/` folder
- `CLAUDE.md` with personality and JSON response format
- `memories.json` with initial memories
- Bear responds with structured JSON when asked "What do you do next?"

## What's Changing

Previously planned: NPCs directly execute actions when triggered
Now: Intent/GM system where NPCs submit what they want to do, GM decides what actually happens

## MVP Implementation Steps (From Here)

### 1. Add Contract Tracking to Game State (30 mins)

**Goal**: Server can track active interactions

Add to existing types:

```rust
// In GameState
contracts: HashMap<String, Contract>,

// In Npc  
active_contract: Option<String>,

// New structs
struct Contract {
    id: String,
    participants: Vec<String>,
    transcript_file: String,
}

struct Intent {
    npc: String,
    thought: String,
    action: String,
    dialogue: Option<String>,
}
```

### 2. Create GM Agent (45 mins)

**Goal**: GM that can resolve simultaneous intents

- Create `/gm/CLAUDE.md` with instructions
- GM takes multiple intents and decides reality
- Start simple: just describe what happens

### 3. Intent Collection Endpoint (30 mins)

**Goal**: Collect intents instead of direct actions

```rust
POST /turn/collect
- Call Claude for each NPC without active contract
- Collect all intents
- Store in pending_intents
```

### 4. GM Resolution Endpoint (45 mins)

**Goal**: GM processes intents

```rust
POST /turn/resolve
- Send all pending_intents to GM
- GM returns what actually happened
- Update game state based on GM response
- Clear pending_intents
```

### 5. Manual Turn Execution (15 mins)

**Goal**: Chain collect + resolve for testing

```rust
POST /turn/execute
- Calls /turn/collect then /turn/resolve
- Returns what happened this turn
```

### 6. Basic Contract Support (1 hour)

**Goal**: Handle simple conversations

- GM creates contracts based on intents (no accept/decline)
- Locked NPCs get contract-specific prompts
- Either can exit by expressing it in their action
- Cleanup triggers memory summarization

### 7. Automated Loop (15 mins)

**Goal**: Game runs itself

```rust
// Every 15 seconds, call execute_turn()
```

## Testing Strategy

```bash
# Current (works now)
curl http://localhost:3000/state

# New endpoints to build
curl -X POST http://localhost:3000/turn/execute
```

## What This Gives You

- ✅ NPCs that think independently
- ✅ GM resolves conflicts (Bear wants to fish, Wolf wants to talk)
- ✅ Basic conversations between NPCs
- ✅ Foundation for richer interactions

## What's NOT in MVP

- ❌ Memory persistence between server restarts
- ❌ Complex contract types (just conversations)
- ❌ Manual intent submission (only automated turns)
- ❌ Environmental events
- ❌ WebSockets/real-time updates

## Key Differences from Original Plan

1. **Intent System**: NPCs express desires naturally, GM interprets into state changes
2. **GM Agent**: New component that resolves simultaneous actions
3. **Contracts**: Simple bindings - exist or don't, no complex states
4. **Turn-based**: Explicit turn execution vs continuous simulation
5. **Simplified Types**: No pending_intents storage, no contract types/states

