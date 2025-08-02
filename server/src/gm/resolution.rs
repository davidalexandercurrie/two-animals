use crate::llm::{parser, LlmClient};
use crate::game::{contracts::ContractManager, GameStateManager};
use crate::prompts::PromptBuilder;
use crate::types::{CurrentState, GmInput, GmResponse, Intent};
use crate::utils::wrap_text;
use anyhow::Result;
use std::sync::Arc;

pub async fn resolve_intents(
    game_manager: &GameStateManager,
    intents: Vec<Intent>,
    llm_client: Arc<dyn LlmClient>,
    prompt_builder: &PromptBuilder,
) -> Result<GmResponse> {
    let intent_count = intents.len();
    log::debug!("Resolving {intent_count} intents with GM");

    // Get current game state
    let game_state = game_manager.get_state();

    // Prepare input for GM
    let gm_input = GmInput {
        current_state: CurrentState {
            npcs: game_state.npcs.clone(),
            active_contracts: game_state.contracts.clone(),
        },
        intents,
    };

    let input_json = serde_json::to_string_pretty(&gm_input)?;
    log::debug!("Sending to GM:\n{input_json}");

    // Build GM prompt using the prompt builder
    let prompt = prompt_builder.build_gm_prompt(&input_json)?;
    log::debug!("GM prompt length: {} chars", prompt.len());
    log::trace!("Full GM prompt:\n{}", prompt);

    // Call GM - use current directory (server directory)
    log::info!("\n🎲 [GM Resolution][GM] Resolving simultaneous actions...");
    let response = llm_client
        .query(prompt, std::path::Path::new("."))
        .await?;

    log::debug!("GM raw response: {response}");

    // Parse response
    let gm_response: GmResponse = parser::extract_json(&response)?;
    let wrapped_reality = wrap_text(&gm_response.reality, 70, "  ");
    log::info!("🎭 [GM Reality][GM]\n{}\n", wrapped_reality);

    // Apply state changes
    for change in &gm_response.state_changes {
        game_manager.update_npc_location(
            &change.npc,
            change.location.clone(),
            change.activity.clone(),
        );
        let npc = &change.npc;
        let location = &change.location;
        let activity = &change.activity;
        log::info!("  📍 [{}] {:?} - {}", npc.to_uppercase(), location, activity);
    }

    // Handle contract updates
    for contract_update in &gm_response.contracts {
        match contract_update.action.as_str() {
            "create" => {
                let contract = ContractManager::create_contract(
                    contract_update.participants.clone(),
                    contract_update.transcript_entry.clone(),
                )?;
                
                // Update NPCs' active_contract field
                for participant in &contract_update.participants {
                    game_manager.set_npc_contract(participant, Some(contract.id.clone()));
                }
                
                // Add to game state
                game_manager.add_contract(contract.clone());
                
                let participants = &contract.participants;
                log::info!("  📝 [Contract] New interaction: {} ↔ {}", participants[0], participants.get(1).unwrap_or(&"self".to_string()));
            }
            "update" => {
                if let Some(contract) = game_manager.get_contract(&contract_update.id) {
                    if let Some(entry) = &contract_update.transcript_entry {
                        ContractManager::update_contract(&contract, entry.clone())?;
                        let id = &contract_update.id;
                        log::debug!("Updated contract {id}");
                    }
                }
            }
            "end" => {
                // Remove active_contract from NPCs
                if let Some(contract) = game_manager.get_contract(&contract_update.id) {
                    for participant in &contract.participants {
                        game_manager.set_npc_contract(participant, None);
                    }
                    let id = &contract_update.id;
                    log::info!("  ✅ [Contract] Interaction ended: {id}");
                }
            }
            _ => {
                let action = &contract_update.action;
                log::error!("Unknown contract action: {action}");
            }
        }
    }

    // Store next prompts from GM
    for (npc_name, prompt) in &gm_response.next_prompts {
        game_manager.set_npc_prompt(npc_name, prompt.clone());
        log::debug!("Stored prompt for {npc_name}");
    }

    Ok(gm_response)
}