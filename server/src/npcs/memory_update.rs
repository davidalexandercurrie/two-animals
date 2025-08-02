use crate::llm::{parser, LlmClient};
use crate::npcs::memory::{MemorySystem, MemoryUpdate};
use crate::prompts::PromptBuilder;
use crate::types::MemoryUpdateInput;
use crate::utils::wrap_text;
use anyhow::Result;
use std::sync::Arc;

pub async fn update_memories(
    memory_inputs: Vec<MemoryUpdateInput>,
    llm_client: Arc<dyn LlmClient>,
    prompt_builder: &PromptBuilder,
) -> Result<()> {
    let total_npcs = memory_inputs.len();
    log::debug!("Updating memories for {total_npcs} NPCs");

    // Update memories sequentially to avoid interleaved logs
    for input in memory_inputs {
        if let Err(e) = update_single_npc_memory(
            input,
            Arc::clone(&llm_client),
            prompt_builder
        ).await {
            log::error!("Memory update failed: {e}");
        }
    }

    Ok(())
}

async fn update_single_npc_memory(
    input: MemoryUpdateInput,
    llm_client: Arc<dyn LlmClient>,
    prompt_builder: &PromptBuilder,
) -> Result<()> {
    let npc_name = &input.npc_name;
    log::debug!("Updating memories for {npc_name}");

    // Load current memories
    let current_memories = load_npc_memories(npc_name)?;

    // Build memory update prompt
    let prompt = prompt_builder.build_memory_update_prompt(
        &input,
        &current_memories
    )?;

    // Query LLM
    log::info!("\n>>> Memory Update: {}\n{}", npc_name.to_uppercase(), "-".repeat(40));
    let working_dir = std::path::Path::new("../data");
    let response = llm_client.query(prompt, working_dir).await?;

    // Parse memory update
    let memory_update: MemoryUpdate = parser::extract_json(&response)?;

    // Apply updates to memory system
    let updated_memories = apply_memory_update(
        current_memories,
        memory_update,
        &input
    )?;

    // Save updated memories
    save_npc_memories(npc_name, &updated_memories)?;

    log::info!("{}\n", "-".repeat(40));
    Ok(())
}

fn load_npc_memories(npc_name: &str) -> Result<MemorySystem> {
    let memory_path = std::path::Path::new("../data/npcs")
        .join(npc_name)
        .join("memories.json");
    
    if memory_path.exists() {
        let content = std::fs::read_to_string(&memory_path)?;
        let memories: MemorySystem = serde_json::from_str(&content)?;
        Ok(memories)
    } else {
        // Initialize empty memory system
        log::debug!("Creating new memory system for {}", npc_name);
        Ok(MemorySystem {
            self_memories: crate::npcs::memory::SelfMemories {
                immediate_context: String::new(),
                recent_events: Vec::new(),
                core_memories: Vec::new(),
            },
            relationships: std::collections::HashMap::new(),
        })
    }
}

fn save_npc_memories(npc_name: &str, memories: &MemorySystem) -> Result<()> {
    let npc_dir = std::path::Path::new("../data/npcs").join(npc_name);
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&npc_dir)?;
    
    let memory_path = npc_dir.join("memories.json");
    
    let json = serde_json::to_string_pretty(memories)?;
    std::fs::write(memory_path, json)?;
    
    Ok(())
}

fn apply_memory_update(
    mut current: MemorySystem,
    update: MemoryUpdate,
    _input: &MemoryUpdateInput,
) -> Result<MemorySystem> {
    // Update self memories
    current.self_memories.immediate_context = update.immediate_self_context.clone();
    let wrapped_state = wrap_text(&update.immediate_self_context, 70, "    ");
    log::info!("  🎭 [Current State]\n{}", wrapped_state);
    
    if let Some(new_event) = update.new_self_memory {
        let wrapped_memory = wrap_text(&new_event, 70, "    ");
        log::info!("  📝 [Personal Memory]\n{}", wrapped_memory);
        current.self_memories.recent_events.push(new_event);
        // Keep only last 10 events
        if current.self_memories.recent_events.len() > 10 {
            let fading = current.self_memories.recent_events.remove(0);
            log::info!("  🌫️ [Memory Fades] {}", fading);
        }
    }

    // Update relationship memories
    for (other_npc, rel_update) in update.relationship_updates {
        let relationship = current.relationships
            .entry(other_npc.clone())
            .or_insert_with(|| crate::npcs::memory::RelationshipMemory {
                immediate_context: String::new(),
                recent_memories: Vec::new(),
                long_term_summary: format!("Just met {}", other_npc),
                core_memories: Vec::new(),
                current_sentiment: 0.0,
                overall_bond: 0.0,
            });

        // Update immediate context
        relationship.immediate_context = rel_update.immediate_context.clone();
        relationship.current_sentiment = rel_update.current_sentiment;
        
        if !rel_update.immediate_context.is_empty() {
            log::info!("\n  🔄 [Relationship with {}]", other_npc.to_uppercase());
            let wrapped_context = wrap_text(&rel_update.immediate_context, 66, "      ");
            log::info!("    - Context:\n{}", wrapped_context);
        }
        
        if rel_update.current_sentiment != 0.0 {
            let sentiment_desc = if rel_update.current_sentiment > 0.0 { "positive" } else { "negative" };
            log::info!("    - Sentiment: {} ({})", sentiment_desc, rel_update.current_sentiment);
        }

        // Add new memory if provided
        if let Some(new_memory) = rel_update.new_memory {
            let wrapped_mem = wrap_text(&new_memory.event, 66, "      ");
            log::info!("    - New memory:\n{}", wrapped_mem);
            relationship.recent_memories.push(new_memory);
            
            // Handle memory limit (10 memories)
            if relationship.recent_memories.len() > 10 {
                // Memory needs to fade
                let fading_memory = relationship.recent_memories.remove(0);
                let wrapped_fade = wrap_text(&fading_memory.event, 66, "      ");
                log::info!("    - Fading memory:\n{}", wrapped_fade);
                
                // Apply long-term summary update if LLM decided it's needed
                if let Some(new_summary) = rel_update.long_term_summary_update {
                    let wrapped_summary = wrap_text(&new_summary, 66, "      ");
                    log::info!("    - Long-term view:\n{}", wrapped_summary);
                    relationship.long_term_summary = new_summary;
                }
            }
        }

        // Handle potential core memory formation
        if let Some(core_memory) = rel_update.potential_core_memory {
            if !relationship.core_memories.contains(&core_memory) {
                relationship.core_memories.push(core_memory.clone());
                let wrapped_core = wrap_text(&core_memory, 66, "      ");
                log::info!("    ✨ Core memory formed:\n{}", wrapped_core);
            }
        }
    }

    Ok(current)
}