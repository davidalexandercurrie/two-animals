# Two Animals - Interaction System

## Overview

The interaction system handles how NPCs act simultaneously and interact with each other in a believable way. Since both NPCs take actions at the same time, we need a system to resolve conflicts and manage interactions.

## The Intent System

Instead of NPCs directly executing actions, they submit **intents** - what they want to do:

- Bear: "I want to go fishing at the river"
- Wolf: "I want to talk to Bear about the territory"

These intents are processed by a GM (Game Master) agent that determines what actually happens.

## The GM Agent

The GM acts as a reality arbiter, resolving simultaneous intents into coherent outcomes:

**Example Resolution:**

- Bear intent: "Run away from the clearing"
- Wolf intent: "Say 'Hey Bear!' to Bear"
- GM output: "As Bear turns to leave, Wolf calls out 'Hey Bear!' Bear is now faced with a choice: stop and talk, or keep running?"

## Contracts

A contract is a binding agreement between participants (NPCs, locations, or activities). Contracts ensure coherent multi-turn interactions by locking participants into a shared context.

### What Contracts Are

A contract is simply a binding between participants. The GM decides what type of interaction it represents based on the intents - could be a conversation, a chase, fishing together, etc. We don't categorize them.

### Contract Lifecycle

1. **GM Creation**: Based on intents, GM creates contract
2. **Immediate Active**: Both parties are in it (no declining)
3. **Interaction**: Parties respond to contract context
4. **Exit**: Either party can leave with their action
5. **Cleanup**: Memories saved, NPCs freed

### Conversation Example

```
1. Wolf initiates: "Talk to Bear"
2. GM creates pending contract
3. Bear's next turn: "Wolf wants to talk. Do you engage?"
4. If yes: Alternating turns until someone ends it
5. Both write their own memory of what happened
```

### NPCs in Contracts Still Have Agency

When in a contract, NPCs aren't just responding mechanically. They still:
- Have internal thoughts and feelings
- Submit intents (speak, listen, gesture, leave)
- Take observable actions beyond just dialogue

Example contract response:
```json
{
  "thought": "He seems angrier than usual",
  "action": "Back away slowly while maintaining eye contact",
  "dialogue": null
}
```

The GM uses all of this to craft the next prompt and update the shared reality.

## Turn Flow

### Complete Turn Execution

1. **Process Active Contracts**
   - For each NPC in a contract: prompt with contract context
   - Example: "Wolf just growled 'Get out of my territory' while blocking the path. How do you respond?"
   - NPCs return full intents (not just dialogue) including thoughts and actions
   - GM processes these to determine what happens and craft next prompts
   - Contract transcript updated with observable actions

2. **Process Free NPCs**
   - For each NPC not in contract: "What do you do next?"
   - NPCs return intents (what they want to do)
   - Collect all intents for GM processing

3. **GM Resolution**
   - GM processes all free NPC intents simultaneously
   - Resolves conflicts and determines reality
   - May create new contracts (e.g., starting conversations)

4. **Contract Cleanup**
   - Check for contracts marked as ending
   - Trigger memory summarization for each participant
   - Archive contract transcript
   - Free NPCs for next turn

## Shared Truth vs Personal Memory

- **Shared Truth**: What actually happened (GM's decision)
- **Personal Memory**: How each NPC remembers it

This allows for different perspectives on the same event, creating richer storytelling.

## Contract Tracking

NPCs track their contract status:
```rust
struct Npc {
    // ... other fields
    active_contract: Option<String>,  // Contract ID if locked
}
```

When in a contract:
- NPC cannot take independent actions
- Must respond to contract context  
- Can choose to exit with their action (e.g., "I turn and leave")
- GM handles the exit and cleanup

## Memory System

### Memory Layers

1. **Immediate Context**
   - Current contract or situation
   - What just happened this turn
   - Temporary, overwritten each turn

2. **Short-term Memory**
   - Recent events (last 5-10 turns)
   - Recent conversations
   - Gradually fades to long-term or forgotten

3. **Long-term Memory**
   - Significant events
   - Important relationships
   - Core experiences that shape behavior

4. **Motivations**
   - Drives created by experiences
   - Basic: hungry, tired, curious
   - Complex: "Find out why ranger stopped visiting"
   - Emotional: "Upset with Wolf after argument"
   - Influence NPC decisions when free

### Contract Memory Processing

When a contract ends:
1. Each participant reads the transcript
2. Claude prompted: "What do you remember from this conversation?"
3. NPC writes personal summary to memories
4. May create new motivations based on interaction
5. Updates relationship status with other participant

