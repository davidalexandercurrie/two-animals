use crate::llm::{parser, LlmClient};
use crate::game::GameStateManager;
use crate::prompts::PromptBuilder;
use crate::types::{Intent, Npc};
use crate::utils::wrap_text;
use futures::future::join_all;
use std::sync::Arc;

pub async fn collect_intents(
    game_manager: &GameStateManager,
    llm_client: Arc<dyn LlmClient>,
    prompt_builder: &PromptBuilder,
) -> Vec<Intent> {
    let npcs_to_process = {
        let game = game_manager.state.lock().unwrap();
        game.npcs.clone()
    };

    let total_npcs = npcs_to_process.len();
    log::debug!("Collecting intents from {total_npcs} NPCs in parallel");

    // Get current game state for prompt building
    let game_state = game_manager.get_state();
    
    // Create futures for all NPCs
    let intent_futures: Vec<_> = npcs_to_process
        .iter()
        .map(|(name, npc)| {
            let name_clone = name.clone();
            let npc_clone = npc.clone();
            let game_state_clone = game_state.clone();
            let llm_client_clone = Arc::clone(&llm_client);
            let prompt_builder_ref = prompt_builder;
            
            async move {
                collect_single_intent(
                    name_clone, 
                    npc_clone, 
                    game_state_clone, 
                    llm_client_clone,
                    prompt_builder_ref
                ).await
            }
        })
        .collect();

    // Wait for all intents to be collected in parallel
    let results = join_all(intent_futures).await;

    // Filter out None values and collect successful intents
    let intents: Vec<Intent> = results.into_iter().flatten().collect();

    let intent_count = intents.len();
    log::debug!("All intents collected! Got {intent_count} intents");
    
    intents
}

async fn collect_single_intent(
    name: String,
    npc: Npc,
    game_state: crate::types::GameState,
    llm_client: Arc<dyn LlmClient>,
    prompt_builder: &PromptBuilder,
) -> Option<Intent> {
    log::debug!("Getting intent from {name}");

    // Build prompt using the prompt builder
    let prompt = match prompt_builder.build_npc_intent_prompt(&npc, &game_state) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to build prompt for {name}: {e}");
            return None;
        }
    };

    // Query LLM - use data directory for working dir
    log::info!("🎭 [Intent Collection][{}] Gathering intent...", name.to_uppercase());
    let working_dir = std::path::Path::new("../data");
    let response = match llm_client.query(prompt, working_dir).await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to get response from LLM for {name}: {e}");
            return None;
        }
    };

    // Parse response
    match parser::extract_json::<Intent>(&response) {
        Ok(intent) => {
            let wrapped_action = wrap_text(&intent.action, 70, "     ");
            log::info!("  💭 [Intent][{}]\n{}", name.to_uppercase(), wrapped_action);
            Some(intent)
        }
        Err(e) => {
            log::error!("Failed to parse intent from {name}: {e}");
            None
        }
    }
}