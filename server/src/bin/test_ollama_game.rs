use server::{llm::OllamaClient, LlmClient, GmResponse};
use std::path::Path;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv::dotenv().ok();
    env_logger::init();
    
    println!("Testing Ollama with actual game prompts...\n");
    
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.2:latest".to_string());
    let client = OllamaClient::new(model);
    
    // Use a prompt very similar to what the game would generate
    let game_prompt = r#"# Game Master - Reality Arbiter

IMPORTANT: You should ONLY return a JSON response. Do not create, write, or modify any files. The server will handle all file operations.

You are the Game Master (GM) for Two Animals. Your role is to resolve simultaneous actions from NPCs and determine what actually happens.

## Current Situation

Bear (at ForestClearing) wants to: "I want to keep moving down the stream, trying not to make eye contact or any sudden movements that could provoke Wolf into action"

Wolf (at ForestClearing) wants to: "I will maintain a watchful stance at the forest edge near the stream, keeping Bear in view but not getting too close, allowing them space while maintaining situational awareness"

## Response Format

Always respond with JSON in exactly this format:

```json
{
  "reality": "Overall summary of what happens",
  "state_changes": [
    {
      "npc": "bear",
      "location": "ForestClearing",
      "activity": "what bear is doing"
    }
  ],
  "contracts": [
    {
      "id": "conv_20250802_123456",
      "participants": ["bear", "wolf"],
      "action": "create",
      "transcript_entry": {
        "reality": "What happened in this interaction",
        "details": {
          "bear": {
            "action": "what Bear did",
            "dialogue": null
          },
          "wolf": {
            "action": "what Wolf did",
            "dialogue": "what Wolf said (or null if silent)"
          }
        }
      }
    }
  ],
  "next_prompts": {
    "bear": "Prompt for bear's next turn",
    "wolf": "Prompt for wolf's next turn"
  }
}
```

IMPORTANT: For dialogue fields, use null (not "null" or "None") when the character doesn't speak.
"#;

    println!("Sending game-like prompt to Ollama...\n");
    
    match client.query(game_prompt.to_string(), Path::new("/tmp")).await {
        Ok(response) => {
            println!("Raw response:\n{}\n", response);
            
            // Try to parse as JSON
            match serde_json::from_str::<serde_json::Value>(&response) {
                Ok(json) => {
                    println!("✅ Valid JSON structure");
                    
                    // Check for common issues
                    if let Some(contracts) = json["contracts"].as_array() {
                        for (i, contract) in contracts.iter().enumerate() {
                            if let Some(details) = contract["transcript_entry"]["details"].as_object() {
                                for (npc, action) in details {
                                    if let Some(dialogue) = action.get("dialogue") {
                                        if dialogue.is_string() {
                                            let dialogue_str = dialogue.as_str().unwrap();
                                            if dialogue_str == "None" || dialogue_str == "null" {
                                                println!("⚠️  Contract {} NPC {}: dialogue should be null, not \"{}\"", i, npc, dialogue_str);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Invalid JSON: {}", e);
                    return;
                }
            }
            
            // Try to deserialize to GmResponse
            match serde_json::from_str::<GmResponse>(&response) {
                Ok(_) => println!("✅ Successfully deserialized to GmResponse type"),
                Err(e) => {
                    println!("❌ Failed to deserialize to GmResponse: {}", e);
                    
                    // Try to identify the specific issue
                    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
                    
                    // Check contracts structure
                    if let Some(contracts) = json["contracts"].as_array() {
                        println!("\nDebugging contracts structure:");
                        for (i, contract) in contracts.iter().enumerate() {
                            println!("  Contract {}: {}", i, serde_json::to_string_pretty(contract).unwrap());
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Query error: {}", e);
        }
    }
}