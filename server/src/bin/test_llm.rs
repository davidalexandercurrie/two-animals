use server::{llm::{ClaudeClient, OllamaClient}, LlmClient};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv::dotenv().ok();
    env_logger::init();
    
    println!("Testing LLM client directly...");
    
    // Check which provider is configured
    let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| {
        eprintln!("No LLM_PROVIDER set in .env file!");
        eprintln!("Run ./bin/setup-llm.sh first");
        std::process::exit(1);
    });
    
    let client: Arc<dyn LlmClient> = match provider.as_str() {
        "claude" => {
            println!("Using Claude provider");
            Arc::new(ClaudeClient)
        }
        "ollama" => {
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.2:latest".to_string());
            println!("Using Ollama provider with model: {}", model);
            Arc::new(OllamaClient::new(model))
        }
        _ => {
            eprintln!("Unknown provider: {}", provider);
            std::process::exit(1);
        }
    };
    
    // Test simple text response
    println!("\n1. Testing simple text response...");
    let result = client.query(
        "Just respond with: hello".to_string(),
        std::path::Path::new(".")
    ).await;
    
    match result {
        Ok(response) => println!("✅ Success! Response: {}", response),
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test JSON response
    println!("\n2. Testing JSON response...");
    let json_prompt = match provider.as_str() {
        "ollama" => {
            "Respond with ONLY a JSON object: {\"status\": \"ok\", \"message\": \"test successful\"}"
        }
        _ => {
            "Respond with JSON: {\"status\": \"ok\", \"message\": \"test successful\"}"
        }
    };
    
    let result = client.query(
        json_prompt.to_string(),
        std::path::Path::new(".")
    ).await;
    
    match result {
        Ok(response) => {
            println!("✅ Raw response: {}", response);
            // Try to parse as JSON
            match serde_json::from_str::<serde_json::Value>(&response) {
                Ok(json) => println!("✅ Valid JSON: {:?}", json),
                Err(e) => println!("⚠️  Not valid JSON: {}", e),
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
}