use crate::llm::LlmClient;
use crate::game::GameStateManager;
use crate::gm::resolve_intents;
use crate::npcs::{collect_intents, update_memories};
use crate::prompts::PromptBuilder;
use crate::types::{GmResponse, MemoryUpdateInput};
use anyhow::Result;
use std::sync::Arc;

pub async fn execute_turn(
    game_manager: &GameStateManager,
    llm_client: Arc<dyn LlmClient>,
    prompt_builder: &PromptBuilder,
) -> Result<GmResponse> {
    log::info!("\n{}\n🎮 [Turn Execution][System] Starting new turn\n{}", "=".repeat(60), "-".repeat(60));

    // First collect intents
    let intents = collect_intents(game_manager, Arc::clone(&llm_client), prompt_builder).await;

    let intent_count = intents.len();
    log::info!("📝 [Intent Summary][System] Collected {intent_count} NPC intents\n");

    // Then resolve them
    let gm_response = resolve_intents(
        game_manager, 
        intents.clone(), 
        Arc::clone(&llm_client), 
        prompt_builder
    ).await?;

    // Update memories based on what happened
    log::info!("\n{}\n🧠 [Memory Phase][System] Updating NPC memories based on turn events\n{}", "-".repeat(60), "-".repeat(60));
    let memory_updates = build_memory_updates(&intents, &gm_response, game_manager)?;
    update_memories(memory_updates, llm_client, prompt_builder).await?;

    Ok(gm_response)
}

fn build_memory_updates(
    intents: &[crate::types::Intent],
    gm_response: &GmResponse,
    game_manager: &GameStateManager,
) -> Result<Vec<MemoryUpdateInput>> {
    let mut memory_updates = Vec::new();
    
    // Get current game state to know who's where
    let game_state = game_manager.get_state();
    
    // Create memory update for each NPC that acted
    for intent in intents {
        // Find other NPCs at the same location
        let npc_location = game_state.npcs
            .get(&intent.npc)
            .map(|npc| npc.location.clone())
            .unwrap_or(crate::types::Location::ForestClearing);
            
        let other_npcs_present: Vec<String> = game_state.npcs
            .iter()
            .filter(|(name, npc)| {
                name.as_str() != intent.npc.as_str() && npc.location == npc_location
            })
            .map(|(name, _)| name.clone())
            .collect();
        
        memory_updates.push(MemoryUpdateInput {
            npc_name: intent.npc.clone(),
            intent: intent.clone(),
            reality: gm_response.reality.clone(),
            other_npcs_present,
        });
    }
    
    Ok(memory_updates)
}