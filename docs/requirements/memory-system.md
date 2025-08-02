# Two Animals - Memory System

## Overview

The memory system allows NPCs to form, store, and evolve memories based on their experiences. Memories are deeply integrated with relationships - each NPC's memories of another character forms the foundation of their relationship.

## Memory Structure

### 1. Self Memories

Personal memories not tied to specific relationships:

```json
{
  "self_memories": {
    "immediate_context": "What I'm doing/feeling right now",
    "recent_events": [
      "Found new berry patch",
      "Heard strange noises last night"
    ],
    "core_memories": [
      "Mother's death during harsh winter",
      "First successful salmon catch"
    ]
  }
}
```

### 2. Relationship Memories

Each relationship tracks memories at multiple temporal levels:

```json
{
  "relationships": {
    "wolf": {
      "immediate_context": "Just blocked my path, being territorial",
      
      "recent_memories": [
        {
          "event": "Wolf is creeping around making noise",
          "timestamp": "2025-01-02T10:30:00Z",
          "emotional_impact": "annoyed",
          "importance": 0.3
        }
        // ... up to 10 recent memories
      ],
      
      "long_term_summary": "Wolf is territorial and often annoying, but we worked together during the poacher incident. Despite disputes, there's grudging respect.",
      
      "core_memories": [
        "When we fought off the poacher together",
        "The winter Wolf shared their kill with me"
      ],
      
      "current_sentiment": -0.2,  // Current feeling (-1 to 1)
      "overall_bond": 0.4        // Long-term relationship (-1 to 1)
    }
  }
}
```

## Memory Flow

### Immediate → Recent → Long-term → Core

1. **Immediate Context** (1 turn)
   - Overwritten every turn
   - Current situation and feelings
   - Fed by GM's reality description

2. **Recent Memories** (10 memory limit)
   - Specific events with timestamps
   - When 11th memory added, oldest fades
   - Fading memories can influence long-term summary

3. **Long-term Summary** (persistent)
   - Natural language summary of the relationship
   - Updated when memories fade or significant events occur
   - Captures patterns and overall sentiment

4. **Core Memories** (permanent)
   - Formed only from highly significant events
   - Define fundamental nature of relationship
   - Never fade

## Memory Updates

### When Memories Update

1. **After Each Turn**
   - Immediate context always updates
   - New recent memories added if interaction occurred

2. **After Intent vs Reality Resolution**

   ```
   Intent: "Going to fish peacefully"
   Reality: "Wolf blocks your path"
   Memory: Colored by the gap between expectation and outcome
   ```

3. **When Memories Fade** (every 10 interactions)
   - Oldest memory triggers summary review
   - LLM decides if fading memory impacts long-term view

4. **After Contracts End**
   - More extensive memory processing
   - Potential for core memory formation

### Memory Formation Process

```
1. NPC expresses intent
2. GM resolves reality
3. Memory update prompt combines both:
   - "You intended: [action]"
   - "You were thinking: [thought]"  
   - "What happened: [GM reality]"
   - "How do you remember this?"
```

### Example Memory Update

**Bear's Intent:**

```json
{
  "thought": "Just need to catch some fish",
  "action": "Head to favorite fishing spot"
}
```

**GM Reality:**
"Wolf emerges, claiming YOUR fishing spot as theirs"

**Memory Formed:**

```json
{
  "event": "Wolf stole my fishing spot when I was hungry",
  "timestamp": "2025-01-02T10:45:00Z",
  "emotional_impact": "frustrated",
  "importance": 0.7
}
```

## Fading and Integration

### When Memory Limit Reached

When adding 11th recent memory:

1. **Fade Trigger**

   ```
   Oldest memory: "Wolf growled at me near berries"
   
   Review prompt: "This memory is fading: [memory]. 
   Current relationship summary: [summary].
   Does this change your overall view?"
   ```

2. **Possible Outcomes**
   - Memory fades with no impact
   - Summary updates to reflect pattern
   - Rarely: forms new core memory

### Emotional Weighting

- Higher importance memories may resist fading
- Extreme events (importance > 0.9) candidate for core memories
- Emotional impact affects how memories color the relationship

## Conflicting Memories

The LLM naturally handles conflicts through prompting:

```
Recent: "Wolf just helped me escape danger"
Long-term: "Wolf is selfish and territorial"

The NPC might update to: "Wolf is complicated - selfish mostly, 
but capable of surprising loyalty"
```

## Implementation Notes

### Server Responsibilities

- Track memory count
- Manage timestamps
- Trigger fade events
- Store/retrieve memory JSON

### LLM Responsibilities  

- Interpret events into memories
- Assign emotional weight
- Update summaries when fading
- Resolve conflicting impressions
- Form rare core memories

### Memory Persistence

- Store in `data/npcs/{name}/memories.json`
- JSON format for runtime mutability
- Backup before major updates
- Include version for migration

## Future Considerations

1. **Memory Reliability**
   - Memories could become distorted over time
   - Different NPCs remember same event differently

2. **Shared Memories**
   - Some events create "shared core memories"
   - Strengthen bonds between participants

3. **Memory Triggers**
   - Certain situations trigger memory recalls
   - Influence current behavior based on past

4. **Collective Memory**
   - Community-wide memories (the great storm, the poacher)
   - Shape world's history

