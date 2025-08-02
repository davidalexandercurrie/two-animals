use crate::game::contracts::ContractManager;
use crate::npcs::memory::MemorySystem;
use crate::prompts::loader::PromptLoader;
use crate::types::{GameState, Npc, MemoryUpdateInput};
use anyhow::Result;

pub struct PromptBuilder {
    loader: PromptLoader,
}

impl PromptBuilder {
    pub fn new(loader: PromptLoader) -> Self {
        Self { loader }
    }

    pub fn build_npc_intent_prompt(
        &self,
        npc: &Npc,
        game_state: &GameState,
    ) -> Result<String> {
        let mut sections = vec![];

        // 1. Base NPC instructions (response format, etc.)
        sections.push(self.loader.load_npc_base()?);
        
        // 2. Personality
        sections.push(self.loader.load_personality(&npc.name)?);
        
        // 3. Current memories
        let memories = self.loader.load_memories(&npc.name)?;
        sections.push(format!("## Your Current Memories\n\n```json\n{}\n```", memories));
        
        // 4. Current state
        sections.push(self.format_current_state(npc, game_state));
        
        // 5. Contract context if in one
        if let Some(contract_id) = &npc.active_contract {
            if let Ok(transcript) = ContractManager::read_contract_transcript(contract_id) {
                sections.push(self.format_contract_context(&transcript));
            }
        }
        
        // 6. GM's specific prompt or generic "What do you do next?"
        let prompt = npc.next_prompt.as_ref()
            .map(|p| p.clone())
            .unwrap_or_else(|| "What do you do next?".to_string());
        sections.push(prompt);

        Ok(sections.join("\n\n---\n\n"))
    }

    pub fn build_gm_prompt(&self, input_json: &str) -> Result<String> {
        let mut sections = vec![];
        
        // GM base instructions
        sections.push(self.loader.load_gm_base()?);
        
        // Current game state and intents
        sections.push(format!("## Current Input\n\n```json\n{}\n```", input_json));
        
        Ok(sections.join("\n\n---\n\n"))
    }

    fn format_current_state(&self, npc: &Npc, game_state: &GameState) -> String {
        let mut state = String::from("## Current Situation\n\n");
        
        // NPC's own state
        state.push_str(&format!("- You are at: {:?}\n", npc.location));
        state.push_str(&format!("- You are: {}\n", npc.activity));
        
        // Others at same location
        let others_here: Vec<_> = game_state.npcs
            .iter()
            .filter(|(name, other_npc)| {
                name.as_str() != npc.name.as_str() && other_npc.location == npc.location
            })
            .collect();
            
        if !others_here.is_empty() {
            state.push_str("\nAlso here:\n");
            for (other_name, other_npc) in others_here {
                state.push_str(&format!("- {} is {}\n", other_name, other_npc.activity));
            }
        }
        
        state
    }
    
    fn format_contract_context(&self, transcript: &[crate::types::TranscriptEntry]) -> String {
        let mut context = String::from("## Ongoing Interaction\n\n");
        context.push_str("You are currently in an interaction with the following history:\n\n");
        
        for (i, entry) in transcript.iter().enumerate() {
            let turn = i + 1;
            context.push_str(&format!("### Turn {}\n", turn));
            context.push_str(&format!("What happened: {}\n\n", entry.reality));
            
            for (participant, action) in &entry.details {
                context.push_str(&format!("**{}**: {}", participant, action.action));
                if let Some(dialogue) = &action.dialogue {
                    context.push_str(&format!(" - Said: \"{}\"", dialogue));
                }
                context.push_str("\n");
            }
            context.push_str("\n");
        }
        
        context.push_str("Remember: You're continuing this interaction. Respond naturally to what just happened.");
        context
    }

    pub fn build_memory_update_prompt(
        &self,
        input: &MemoryUpdateInput,
        current_memories: &MemorySystem,
    ) -> Result<String> {
        let mut sections = vec![];
        
        // Instructions for memory updates
        sections.push(r#"## Memory Update Task

IMPORTANT: You should ONLY return a JSON response. Do not create, write, or modify any files. The server will handle all file operations.

You need to update your memories based on what just happened. Consider:
- Your intent vs what actually occurred
- Emotional impact and importance of events
- Changes in relationships

Output a JSON object with this structure:
```json
{
  "immediate_self_context": "What I'm doing/feeling right now",
  "new_self_memory": "Optional: significant personal event to remember",
  "relationship_updates": {
    "other_npc_name": {
      "immediate_context": "Current situation with them",
      "new_memory": {
        "event": "What happened",
        "timestamp": "2025-01-02T10:30:00Z",
        "emotional_impact": "frustrated/happy/angry/etc",
        "importance": 0.7
      },
      "current_sentiment": -0.3,
      "long_term_summary_update": "Optional: only if this changes your overall view",
      "potential_core_memory": "Optional: only for truly defining moments"
    }
  }
}
```

Notes:
- importance: 0.0-1.0 (0.9+ for potential core memories)
- current_sentiment: -1.0 to 1.0 (negative=dislike, positive=like)
- Only include relationship_updates for NPCs you interacted with
- Be selective with core memories - they define relationships permanently"#.to_string());

        // Current memory state
        sections.push(format!("## Your Current Memories\n\n```json\n{}\n```", 
            serde_json::to_string_pretty(current_memories)?));
        
        // What happened
        sections.push(format!("## What Just Happened\n\nYou are: {}", input.npc_name));
        sections.push(format!("You intended:\n- Thought: {}\n- Action: {}", 
            input.intent.thought, input.intent.action));
        if let Some(dialogue) = &input.intent.dialogue {
            sections.push(format!("- You wanted to say: \"{}\"", dialogue));
        }
        
        sections.push(format!("\nWhat actually happened:\n{}", input.reality));
        
        if !input.other_npcs_present.is_empty() {
            sections.push(format!("\nOthers present: {}", input.other_npcs_present.join(", ")));
        }
        
        sections.push("\nNow update your memories based on this experience.".to_string());
        
        Ok(sections.join("\n\n---\n\n"))
    }
}