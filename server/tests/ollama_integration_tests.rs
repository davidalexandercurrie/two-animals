#[cfg(test)]
mod ollama_integration_tests {
    use server::{llm::OllamaClient, LlmClient, Intent, GmResponse, npcs::memory::MemoryUpdate};
    use serde_json::Value;
    use std::path::Path;

    // Load .env file for tests
    fn setup() {
        dotenv::dotenv().ok();
    }
    
    // Get the model to test with
    fn get_test_model() -> String {
        std::env::var("TEST_LLM_MODEL")
            .or_else(|_| std::env::var("LLM_MODEL"))
            .unwrap_or_else(|_| "llama3.2:latest".to_string())
    }

    // Only run these tests when explicitly requested with:
    // cargo test --test ollama_integration_tests -- --ignored
    
    #[tokio::test]
    #[ignore] // Ignored by default since it calls real Ollama API
    async fn test_ollama_simple_json() {
        setup();
        
        let model = get_test_model();
        let client = OllamaClient::new(model);
        
        let test_prompt = r#"
Respond with ONLY a JSON object containing a single field 'status' with value 'ok'. 
Example: {"status": "ok"}
"#;

        let response = client.query(test_prompt.to_string(), Path::new("/tmp"))
            .await
            .expect("Ollama query failed");
        
        println!("Response: {}", response);
        
        // Try to parse the response as JSON
        let parsed: Result<Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "Response was not valid JSON: {}", response);
        
        let json = parsed.unwrap();
        assert_eq!(json["status"], "ok", "Status field was not 'ok'");
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_ollama_intent_response_format() {
        setup();
        
        // Initialize env_logger for debugging
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
            
        let model = get_test_model();
        let client = OllamaClient::new(model);
        
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
            .expect("Ollama query failed");
        
        println!("Response: {}", response);
        
        // Try to parse the response as JSON
        let parsed: Result<Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "Response was not valid JSON: {}", response);
        
        let json = parsed.unwrap();
        
        // Verify the structure
        assert!(json["npc"].is_string(), "Missing or invalid 'npc' field");
        assert!(json["thought"].is_string(), "Missing or invalid 'thought' field");
        assert!(json["action"].is_string(), "Missing or invalid 'action' field");
        assert!(json.get("dialogue").is_some(), "Missing 'dialogue' field");
        
        // Try to deserialize to the actual Intent type
        let intent: Result<Intent, _> = serde_json::from_str(&response);
        assert!(intent.is_ok(), "Could not deserialize to Intent type: {}", response);
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_ollama_gm_response_format() {
        setup();
        
        let model = get_test_model();
        let client = OllamaClient::new(model);
        
        // Simplified GM prompt to test the structure
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

Current situation: Bear wants to fish, Wolf wants to hunt. They meet at the ForestClearing.
"#;

        let response = client.query(test_prompt.to_string(), Path::new("/tmp"))
            .await
            .expect("Ollama query failed");
        
        println!("GM Response: {}", response);
        
        let parsed: Result<Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "Response was not valid JSON: {}", response);
        
        let json = parsed.unwrap();
        
        // Verify the structure
        assert!(json["reality"].is_string(), "Missing or invalid 'reality' field");
        assert!(json["state_changes"].is_array(), "Missing or invalid 'state_changes' field");
        assert!(json["contracts"].is_array(), "Missing or invalid 'contracts' field");
        assert!(json["next_prompts"].is_object(), "Missing or invalid 'next_prompts' field");
        
        // Try to deserialize to the actual GmResponse type
        let gm_response: Result<GmResponse, _> = serde_json::from_str(&response);
        if let Err(e) = &gm_response {
            println!("Deserialization error: {}", e);
        }
        assert!(gm_response.is_ok(), "Could not deserialize to GmResponse type: {}", response);
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_ollama_contract_update_format() {
        setup();
        
        let model = get_test_model();
        let client = OllamaClient::new(model);
        
        // Test the complex contract structure that's failing
        let test_prompt = r#"
You are a Game Master. Respond with ONLY a JSON object that includes a contract update:
{
  "reality": "Bear and Wolf meet at the stream",
  "state_changes": [
    {
      "npc": "bear",
      "location": "ForestClearing",
      "activity": "fishing"
    },
    {
      "npc": "wolf",
      "location": "ForestClearing", 
      "activity": "watching"
    }
  ],
  "contracts": [
    {
      "id": "conv_test_123",
      "participants": ["bear", "wolf"],
      "action": "create",
      "transcript_entry": {
        "reality": "They notice each other",
        "details": {
          "bear": {
            "action": "continues fishing cautiously",
            "dialogue": null
          },
          "wolf": {
            "action": "watches from distance",
            "dialogue": "This is my territory"
          }
        }
      }
    }
  ],
  "next_prompts": {
    "bear": "You notice Wolf watching you",
    "wolf": "Bear is in your territory"
  }
}

Ensure the transcript_entry is an object with reality and details fields.
"#;

        let response = client.query(test_prompt.to_string(), Path::new("/tmp"))
            .await
            .expect("Ollama query failed");
        
        println!("Contract Response: {}", response);
        
        let parsed: Result<Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "Response was not valid JSON: {}", response);
        
        // Try to deserialize to the actual GmResponse type
        let gm_response: Result<GmResponse, _> = serde_json::from_str(&response);
        if let Err(e) = &gm_response {
            println!("Deserialization error: {}", e);
            println!("Full response: {}", response);
        }
        assert!(gm_response.is_ok(), "Could not deserialize to GmResponse with contract: {}", response);
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_ollama_memory_response_format() {
        setup();
        
        let model = get_test_model();
        let client = OllamaClient::new(model);
        
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
            .expect("Ollama query failed");
        
        println!("Memory Response: {}", response);
        
        let parsed: Result<Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "Response was not valid JSON: {}", response);
        
        let json = parsed.unwrap();
        
        // Verify the structure
        assert!(json["immediate_self_context"].is_string(), "Missing or invalid 'immediate_self_context'");
        assert!(json.get("new_self_memory").is_some(), "Missing 'new_self_memory' field");
        assert!(json["relationship_updates"].is_object(), "Missing or invalid 'relationship_updates'");
        
        // Try to deserialize to the actual MemoryUpdate type
        let memory_update: Result<MemoryUpdate, _> = serde_json::from_str(&response);
        assert!(memory_update.is_ok(), "Could not deserialize to MemoryUpdate type: {}", response);
    }
}