use server::{llm::ClaudeClient, LlmClient};

#[tokio::main]
async fn main() {
    env_logger::init();
    
    println!("Testing LLM client directly...");
    
    let client = ClaudeClient;
    let result = client.query(
        "Just respond with: hello".to_string(),
        std::path::Path::new(".")
    ).await;
    
    match result {
        Ok(response) => println!("Success! Response: {}", response),
        Err(e) => println!("Error: {}", e),
    }
}