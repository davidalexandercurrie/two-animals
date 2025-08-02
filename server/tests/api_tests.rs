use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use server::{llm::LlmClient, game::GameStateManager, prompts::{PromptBuilder, PromptLoader}};
use std::sync::Arc;
use tower::ServiceExt;
use serde_json::{json, Value};

// Mock LLM client for testing
struct MockLlmClient {
    responses: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl MockLlmClient {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses: std::sync::Arc::new(std::sync::Mutex::new(responses)),
        }
    }
}

#[async_trait::async_trait]
impl LlmClient for MockLlmClient {
    async fn query(&self, _prompt: String, _working_dir: &std::path::Path) -> anyhow::Result<String> {
        let mut responses = self.responses.lock().unwrap();
        if let Some(response) = responses.pop() {
            Ok(response)
        } else {
            Ok(r#"{"thought": "test", "action": "test", "dialogue": null}"#.to_string())
        }
    }
}

// Helper to create test app
async fn create_test_app() -> axum::Router {
    // Create test data directory
    let test_data_dir = std::env::temp_dir().join("two_animals_test");
    std::fs::create_dir_all(&test_data_dir).unwrap();
    
    // Create minimal prompt files
    let prompts_dir = test_data_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir.join("core")).unwrap();
    std::fs::create_dir_all(&prompts_dir.join("gm")).unwrap();
    std::fs::write(prompts_dir.join("core/npc_base.md"), "Test NPC base").unwrap();
    std::fs::write(prompts_dir.join("gm/gm_base.md"), "Test GM base").unwrap();
    
    // Create NPC directories
    let npcs_dir = test_data_dir.join("npcs");
    std::fs::create_dir_all(&npcs_dir.join("bear")).unwrap();
    std::fs::create_dir_all(&npcs_dir.join("wolf")).unwrap();
    std::fs::write(npcs_dir.join("bear/personality.md"), "Test bear personality").unwrap();
    std::fs::write(npcs_dir.join("wolf/personality.md"), "Test wolf personality").unwrap();
    std::fs::write(npcs_dir.join("bear/memories.json"), "{}").unwrap();
    std::fs::write(npcs_dir.join("wolf/memories.json"), "{}").unwrap();
    
    let game_manager = GameStateManager::new();
    let llm_client: Arc<dyn LlmClient> = Arc::new(MockLlmClient::new(vec![]));
    let prompt_loader = PromptLoader::new(test_data_dir);
    let prompt_builder = PromptBuilder::new(prompt_loader);
    
    let app_state = Arc::new(server::AppState {
        game_manager,
        llm_client,
        prompt_builder,
    });
    
    server::create_router(app_state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"OK\n");
}

#[tokio::test]
async fn test_state_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder().uri("/state").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    // Check the structure
    assert!(json["npcs"].is_object());
    assert!(json["contracts"].is_object());
    
    // Check NPCs are initialized
    assert!(json["npcs"]["bear"].is_object());
    assert!(json["npcs"]["wolf"].is_object());
    assert_eq!(json["npcs"]["bear"]["name"], "bear");
    assert_eq!(json["npcs"]["wolf"]["name"], "wolf");
}

#[tokio::test]
async fn test_collect_intents_endpoint() {
    // Create mock responses for bear and wolf
    let bear_response = json!({
        "npc": "bear",
        "thought": "I should go fishing",
        "action": "Head to the river",
        "dialogue": null
    }).to_string();
    
    let wolf_response = json!({
        "npc": "wolf", 
        "thought": "Time to hunt",
        "action": "Patrol the forest",
        "dialogue": "The hunt begins"
    }).to_string();
    
    let mock_client = MockLlmClient::new(vec![wolf_response, bear_response]);
    
    // Create app with mock client
    let test_data_dir = std::env::temp_dir().join("two_animals_test2");
    std::fs::create_dir_all(&test_data_dir).unwrap();
    
    // Set up directories
    let prompts_dir = test_data_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir.join("core")).unwrap();
    std::fs::create_dir_all(&prompts_dir.join("gm")).unwrap();
    std::fs::write(prompts_dir.join("core/npc_base.md"), "Test").unwrap();
    std::fs::write(prompts_dir.join("gm/gm_base.md"), "Test").unwrap();
    
    let npcs_dir = test_data_dir.join("npcs");
    std::fs::create_dir_all(&npcs_dir.join("bear")).unwrap();
    std::fs::create_dir_all(&npcs_dir.join("wolf")).unwrap();
    std::fs::write(npcs_dir.join("bear/personality.md"), "Test").unwrap();
    std::fs::write(npcs_dir.join("wolf/personality.md"), "Test").unwrap();
    std::fs::write(npcs_dir.join("bear/memories.json"), "{}").unwrap();
    std::fs::write(npcs_dir.join("wolf/memories.json"), "{}").unwrap();
    
    let game_manager = GameStateManager::new();
    let prompt_loader = PromptLoader::new(test_data_dir);
    let prompt_builder = PromptBuilder::new(prompt_loader);
    
    let app_state = Arc::new(server::AppState {
        game_manager,
        llm_client: Arc::new(mock_client),
        prompt_builder,
    });
    
    let app = server::create_router(app_state);
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/turn/collect")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let intents: Vec<Value> = serde_json::from_slice(&body).unwrap();
    
    // Should have 2 intents
    assert_eq!(intents.len(), 2);
    
    // Check structure
    for intent in &intents {
        assert!(intent["npc"].is_string());
        assert!(intent["thought"].is_string());
        assert!(intent["action"].is_string());
        assert!(intent.get("dialogue").is_some());
    }
}

#[tokio::test]
async fn test_resolve_intents_endpoint() {
    let gm_response = json!({
        "reality": "Bear and Wolf meet at the river",
        "state_changes": [{
            "npc": "bear",
            "location": "ForestClearing",
            "activity": "fishing by the river"
        }],
        "contracts": [],
        "next_prompts": {
            "bear": "You see Wolf approaching. What do you do?"
        }
    }).to_string();
    
    let mock_client = MockLlmClient::new(vec![gm_response]);
    
    let test_data_dir = std::env::temp_dir().join("two_animals_test3");
    std::fs::create_dir_all(&test_data_dir).unwrap();
    
    let prompts_dir = test_data_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir.join("core")).unwrap();
    std::fs::create_dir_all(&prompts_dir.join("gm")).unwrap();
    std::fs::write(prompts_dir.join("core/npc_base.md"), "Test").unwrap();
    std::fs::write(prompts_dir.join("gm/gm_base.md"), "Test").unwrap();
    
    let game_manager = GameStateManager::new();
    let prompt_loader = PromptLoader::new(test_data_dir);
    let prompt_builder = PromptBuilder::new(prompt_loader);
    
    let app_state = Arc::new(server::AppState {
        game_manager,
        llm_client: Arc::new(mock_client),
        prompt_builder,
    });
    
    let app = server::create_router(app_state);
    
    let test_intents = json!([{
        "npc": "bear",
        "thought": "Need food",
        "action": "Go fishing",
        "dialogue": null
    }]);
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/turn/resolve")
                .header("content-type", "application/json")
                .body(Body::from(test_intents.to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let gm_response: Value = serde_json::from_slice(&body).unwrap();
    
    // Check structure
    assert!(gm_response["reality"].is_string());
    assert!(gm_response["state_changes"].is_array());
    assert!(gm_response["contracts"].is_array());
    assert!(gm_response["next_prompts"].is_object());
}

#[tokio::test]
async fn test_memory_update_endpoint() {
    let memory_response = json!({
        "immediate_self_context": "Just went fishing",
        "new_self_memory": null,
        "relationship_updates": {}
    }).to_string();
    
    let mock_client = MockLlmClient::new(vec![memory_response]);
    
    let test_data_dir = std::env::temp_dir().join("two_animals_test4");
    std::fs::create_dir_all(&test_data_dir).unwrap();
    
    let npcs_dir = test_data_dir.join("npcs");
    std::fs::create_dir_all(&npcs_dir.join("bear")).unwrap();
    std::fs::write(npcs_dir.join("bear/memories.json"), "{}").unwrap();
    
    let game_manager = GameStateManager::new();
    let prompt_loader = PromptLoader::new(test_data_dir);
    let prompt_builder = PromptBuilder::new(prompt_loader);
    
    let app_state = Arc::new(server::AppState {
        game_manager,
        llm_client: Arc::new(mock_client),
        prompt_builder,
    });
    
    let app = server::create_router(app_state);
    
    let memory_updates = json!([{
        "npc_name": "bear",
        "intent": {
            "npc": "bear",
            "thought": "Hungry",
            "action": "Fish",
            "dialogue": null
        },
        "reality": "Bear caught a fish",
        "other_npcs_present": []
    }]);
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/turn/memories")
                .header("content-type", "application/json")
                .body(Body::from(memory_updates.to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "Memories updated");
}

#[tokio::test]
async fn test_execute_full_turn_endpoint() {
    // Initialize env_logger for test output
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
        
    // This test should have caught the memory issue!
    // We need responses for: collect intents (2 NPCs), GM resolution, memory updates (2 NPCs)
    
    let memory_response = json!({
        "immediate_self_context": "Just interacted",
        "new_self_memory": null,
        "relationship_updates": {}
    }).to_string();
    
    let gm_response = json!({
        "reality": "Bear and Wolf meet at the river",
        "state_changes": [{
            "npc": "bear",
            "location": "ForestClearing", 
            "activity": "fishing"
        }],
        "contracts": [],
        "next_prompts": {}
    }).to_string();
    
    let bear_intent = json!({
        "npc": "bear",
        "thought": "Hungry",
        "action": "Fish",
        "dialogue": null
    }).to_string();
    
    let wolf_intent = json!({
        "npc": "wolf",
        "thought": "Prowling",
        "action": "Hunt",
        "dialogue": null
    }).to_string();
    
    // Order matters - they're popped in reverse
    let mock_client = MockLlmClient::new(vec![
        memory_response.clone(), // Wolf memory update
        memory_response,         // Bear memory update  
        gm_response,            // GM resolution
        wolf_intent,            // Wolf intent
        bear_intent,            // Bear intent
    ]);
    
    let test_data_dir = std::env::temp_dir().join("two_animals_test_full_turn");
    std::fs::create_dir_all(&test_data_dir).unwrap();
    
    // Set up all required directories and files
    let prompts_dir = test_data_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir.join("core")).unwrap();
    std::fs::create_dir_all(&prompts_dir.join("gm")).unwrap();
    std::fs::write(prompts_dir.join("core/npc_base.md"), "Test").unwrap();
    std::fs::write(prompts_dir.join("gm/gm_base.md"), "Test").unwrap();
    
    let npcs_dir = test_data_dir.join("npcs");
    std::fs::create_dir_all(&npcs_dir.join("bear")).unwrap();
    std::fs::create_dir_all(&npcs_dir.join("wolf")).unwrap();
    std::fs::write(npcs_dir.join("bear/personality.md"), "Test bear").unwrap();
    std::fs::write(npcs_dir.join("wolf/personality.md"), "Test wolf").unwrap();
    
    // This is where the test would have failed with old memory format!
    // Let's create old format memory files to reproduce the issue
    std::fs::write(npcs_dir.join("bear/memories.json"), r#"{
        "recent_thoughts": ["The berries are ripening"],
        "important_memories": ["Last winter was harsh"],
        "relationships": {
            "wolf": "neutral"
        }
    }"#).unwrap();
    
    std::fs::write(npcs_dir.join("wolf/memories.json"), r#"{
        "recent_thoughts": ["Morning patrol"],
        "important_memories": ["The pack moved south"],
        "relationships": {
            "bear": "territorial rival"
        }
    }"#).unwrap();
    
    let game_manager = GameStateManager::new();
    let prompt_loader = PromptLoader::new(test_data_dir);
    let prompt_builder = PromptBuilder::new(prompt_loader);
    
    let app_state = Arc::new(server::AppState {
        game_manager,
        llm_client: Arc::new(mock_client),
        prompt_builder,
    });
    
    let app = server::create_router(app_state);
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/turn/execute")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let turn_result: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify the full turn completed
    assert!(turn_result["reality"].is_string());
    assert!(turn_result["state_changes"].is_array());
}