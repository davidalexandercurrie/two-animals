pub mod llm;
pub mod game;
pub mod gm;
pub mod logging;
pub mod npcs;
pub mod prompts;
pub mod types;
pub mod utils;

// Re-export for tests
pub use llm::{ClaudeClient, LlmClient};
pub use game::GameStateManager;
pub use prompts::{PromptBuilder, PromptLoader};
pub use types::*;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
// Using crate:: to avoid shadowing the public export

pub struct AppState {
    pub game_manager: GameStateManager,
    pub llm_client: Arc<dyn LlmClient>,
    pub prompt_builder: PromptBuilder,
}

type SharedState = Arc<AppState>;

async fn health() -> &'static str {
    "OK\n"
}

async fn get_game_state(State(state): State<SharedState>) -> Json<types::GameState> {
    Json(state.game_manager.get_state())
}

async fn collect_intents_handler(State(state): State<SharedState>) -> Json<Vec<types::Intent>> {
    let intents = npcs::collect_intents(
        &state.game_manager, 
        Arc::clone(&state.llm_client),
        &state.prompt_builder
    ).await;
    Json(intents)
}

async fn resolve_intents_handler(
    State(state): State<SharedState>,
    Json(intents): Json<Vec<types::Intent>>,
) -> Json<types::GmResponse> {
    match gm::resolve_intents(
        &state.game_manager, 
        intents, 
        Arc::clone(&state.llm_client),
        &state.prompt_builder
    ).await {
        Ok(response) => Json(response),
        Err(e) => {
            log::error!("Failed to resolve intents: {e}");
            panic!("GM resolution failed");
        }
    }
}

#[axum::debug_handler]
async fn update_memories_handler(
    State(state): State<SharedState>,
    Json(memory_updates): Json<Vec<types::MemoryUpdateInput>>,
) -> String {
    match npcs::update_memories(
        memory_updates,
        Arc::clone(&state.llm_client),
        &state.prompt_builder
    ).await {
        Ok(_) => "Memories updated".to_string(),
        Err(e) => {
            log::error!("Failed to update memories: {e}");
            panic!("Memory update failed");
        }
    }
}

async fn execute_turn_handler(
    State(state): State<SharedState>,
    Json(request): Json<types::ExecuteTurnRequest>,
) -> Json<types::ExecuteTurnResponse> {
    let repeat_count = request.repeat.unwrap_or(1);
    let mut turns_executed = 0;
    let mut last_result = None;
    
    if request.endless {
        log::info!("🔄 [Turn Mode][System] Starting endless turn execution (delay: {}ms)", request.delay_ms);
        loop {
            match game::turn::execute_turn(
                &state.game_manager,
                Arc::clone(&state.llm_client),
                &state.prompt_builder
            ).await {
                Ok(response) => {
                    turns_executed += 1;
                    last_result = Some(response);
                    log::info!("\n✅ [Turn Complete][System] Turn {} completed\n{}\n", turns_executed, "=".repeat(60));
                    tokio::time::sleep(tokio::time::Duration::from_millis(request.delay_ms)).await;
                }
                Err(e) => {
                    log::error!("Failed to execute turn {}: {e}", turns_executed + 1);
                    break;
                }
            }
        }
    } else {
        log::info!("🔄 [Turn Mode][System] Executing {} turn(s)", repeat_count);
        for i in 0..repeat_count {
            match game::turn::execute_turn(
                &state.game_manager,
                Arc::clone(&state.llm_client),
                &state.prompt_builder
            ).await {
                Ok(response) => {
                    turns_executed += 1;
                    last_result = Some(response);
                    log::info!("\n✅ [Turn Complete][System] Turn {}/{} completed\n{}\n", turns_executed, repeat_count, "=".repeat(60));
                    
                    // Add delay between turns if not the last turn
                    if i < repeat_count - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(request.delay_ms)).await;
                    }
                }
                Err(e) => {
                    log::error!("Failed to execute turn {}: {e}", i + 1);
                    break;
                }
            }
        }
    }
    
    Json(types::ExecuteTurnResponse {
        turns_executed,
        last_turn_result: last_result,
        status: if request.endless {
            "Endless mode stopped".to_string()
        } else {
            format!("Executed {}/{} turns", turns_executed, repeat_count)
        },
    })
}

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/state", get(get_game_state))
        .route("/turn/collect", post(collect_intents_handler))
        .route("/turn/resolve", post(resolve_intents_handler))
        .route("/turn/memories", post(update_memories_handler))
        .route("/turn/execute", post(execute_turn_handler))
        .with_state(app_state)
}