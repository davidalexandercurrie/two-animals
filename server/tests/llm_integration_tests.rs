#[cfg(test)]
mod llm_integration_tests {
    use server::{llm::{ClaudeClient, OllamaClient}, LlmClient};
    use serde_json::Value;
    use std::path::Path;
    use std::sync::Arc;

    // Only run these tests when explicitly requested with:
    // cargo test --test llm_integration_tests -- --ignored
    
    fn get_llm_client() -> Arc<dyn LlmClient> {
        // Check for test-specific provider first
        let provider = std::env::var("TEST_LLM_PROVIDER")
            .or_else(|_| {
                // Load .env file if no test provider specified
                dotenv::dotenv().ok();
                std::env::var("LLM_PROVIDER")
            })
            .unwrap_or_else(|_| "claude".to_string());
        
        match provider.as_str() {
            "claude" => Arc::new(ClaudeClient),
            "ollama" => {
                let model = std::env::var("TEST_LLM_MODEL")
                    .or_else(|_| std::env::var("LLM_MODEL"))
                    .unwrap_or_else(|_| "llama3.2:latest".to_string());
                Arc::new(OllamaClient::new(model))
            }
            _ => panic!("Unknown LLM provider: {}", provider),
        }
    }
    
    #[tokio::test]
    #[ignore] // Ignored by default since it calls real LLM API
    async fn test_llm_intent_response_format() {
        // Initialize env_logger for debugging
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
            
        let client = get_llm_client();
        
        let test_prompt = r#"
You are a bear in a forest. Respond with ONLY a JSON object in this exact format:
{
  "npc": "bear",
  "thought": "your internal thought",
  "action": "what you do",
  "dialogue": "what you say (or null if silent)"
}

Current situation: You are hungry and want to find food in the forest.
What do you do?
"#;

        let response = client.query(test_prompt.to_string(), Path::new("/tmp"))
            .await
            .expect("LLM query failed");
        
        // Try to parse the response as JSON
        let parsed: Result<Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "LLM response was not valid JSON: {}", response);
        
        let json = parsed.unwrap();
        
        // Verify the structure
        assert!(json["npc"].is_string(), "Missing or invalid 'npc' field");
        assert!(json["thought"].is_string(), "Missing or invalid 'thought' field");
        assert!(json["action"].is_string(), "Missing or invalid 'action' field");
        assert!(json.get("dialogue").is_some(), "Missing 'dialogue' field");
        
        // Could also deserialize to the actual Intent type
        let intent: Result<server::Intent, _> = serde_json::from_str(&response);
        assert!(intent.is_ok(), "Could not deserialize to Intent type: {}", response);
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_llm_gm_response_format() {
        let client = get_llm_client();
        
        let test_prompt = r#"
You are a Game Master. Respond with ONLY a JSON object in this exact format:
{
  "reality": "what actually happens",
  "state_changes": [
    {
      "npc": "character name",
      "location": "ForestClearing",
      "activity": "what they're doing"
    }
  ],
  "contracts": [],
  "next_prompts": {}
}

Current situation: Bear wants to fish, Wolf wants to hunt. They meet at the DeepForest.
"#;

        let response = client.query(test_prompt.to_string(), Path::new("/tmp"))
            .await
            .expect("LLM query failed");
        
        let parsed: Result<Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "LLM response was not valid JSON: {}", response);
        
        let json = parsed.unwrap();
        
        // Verify the structure
        assert!(json["reality"].is_string(), "Missing or invalid 'reality' field");
        assert!(json["state_changes"].is_array(), "Missing or invalid 'state_changes' field");
        assert!(json["contracts"].is_array(), "Missing or invalid 'contracts' field");
        assert!(json["next_prompts"].is_object(), "Missing or invalid 'next_prompts' field");
        
        // Could also deserialize to the actual GmResponse type
        let gm_response: Result<server::GmResponse, _> = serde_json::from_str(&response);
        assert!(gm_response.is_ok(), "Could not deserialize to GmResponse type: {}", response);
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_llm_memory_response_format() {
        let client = get_llm_client();
        
        let test_prompt = r#"
You need to update memories. Respond with ONLY a JSON object in this exact format:
{
  "immediate_self_context": "current situation",
  "new_self_memory": null,
  "relationship_updates": {
    "other_npc": {
      "immediate_context": "what happened with them",
      "new_memory": null,
      "current_sentiment": -0.2,
      "long_term_summary_update": null,
      "potential_core_memory": null
    }
  }
}

You are: bear
You intended to fish but wolf blocked your path.
"#;

        let response = client.query(test_prompt.to_string(), Path::new("/tmp"))
            .await
            .expect("LLM query failed");
        
        let parsed: Result<Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "LLM response was not valid JSON: {}", response);
        
        let json = parsed.unwrap();
        
        // Verify the structure
        assert!(json["immediate_self_context"].is_string(), "Missing or invalid 'immediate_self_context'");
        assert!(json.get("new_self_memory").is_some(), "Missing 'new_self_memory' field");
        assert!(json["relationship_updates"].is_object(), "Missing or invalid 'relationship_updates'");
        
        // Could also deserialize to the actual MemoryUpdate type
        let memory_update: Result<server::npcs::memory::MemoryUpdate, _> = serde_json::from_str(&response);
        assert!(memory_update.is_ok(), "Could not deserialize to MemoryUpdate type: {}", response);
    }
}