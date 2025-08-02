use std::sync::Arc;
use server::{
    AppState, ClaudeClient, OllamaClient, GameStateManager, LlmClient, PromptBuilder, PromptLoader,
    create_router, logging,
};

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    dotenv::dotenv().ok();
    
    // Initialize logger
    logging::init_logger();
    
    let addr = "0.0.0.0:3000";

    // Initialize game state
    let game_manager = GameStateManager::new();
    
    // Initialize LLM client based on environment variable
    let llm_provider = match std::env::var("LLM_PROVIDER") {
        Ok(provider) => provider,
        Err(_) => {
            log::error!("❌ [Server][LLM] No LLM provider configured!");
            log::error!("");
            log::error!("Please run the setup script first:");
            log::error!("  ./bin/setup-llm.sh");
            log::error!("");
            log::error!("This will help you choose and configure an LLM provider.");
            log::error!("Available providers: claude, ollama");
            std::process::exit(1);
        }
    };

    let llm_client: Arc<dyn LlmClient> = match llm_provider.as_str() {
        "claude" => {
            log::info!("🤖 [Server][LLM] Using Claude CLI provider");
            
            // Check if Claude CLI is available
            let claude_check = tokio::process::Command::new("which")
                .arg("claude")
                .output()
                .await;
                
            if claude_check.is_err() || !claude_check.unwrap().status.success() {
                log::error!("❌ [Server][LLM] Claude CLI not found!");
                log::error!("");
                log::error!("Please install Claude CLI first:");
                log::error!("  Visit: https://docs.anthropic.com/en/docs/claude-cli");
                log::error!("");
                log::error!("Or run ./bin/setup-llm.sh to choose a different provider.");
                std::process::exit(1);
            }
            
            if let Ok(model) = std::env::var("LLM_MODEL") {
                log::info!("🤖 [Server][LLM] Model: {}", model);
            }
            Arc::new(ClaudeClient)
        }
        "ollama" => {
            log::info!("🤖 [Server][LLM] Using Ollama provider");
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.2:latest".to_string());
            log::info!("🤖 [Server][LLM] Model: {}", model);
            
            // Check if Ollama is running
            if let Err(_) = server::llm::ollama::check_ollama_status("http://localhost:11434").await {
                log::error!("❌ [Server][LLM] Ollama is not running!");
                log::error!("");
                log::error!("Please start Ollama service first:");
                log::error!("  sudo systemctl start ollama");
                log::error!("");
                log::error!("To check if Ollama is running:");
                log::error!("  sudo systemctl status ollama");
                log::error!("");
                log::error!("If Ollama is not installed, run:");
                log::error!("  ./bin/setup-llm.sh");
                std::process::exit(1);
            }
            
            log::info!("✅ [Server][LLM] Ollama is running");
            Arc::new(OllamaClient::new(model))
        }
        provider => {
            log::error!("❌ [Server][LLM] Unknown provider: {}", provider);
            log::error!("");
            log::error!("Supported providers: claude, ollama");
            log::error!("");
            log::error!("Please run ./bin/setup-llm.sh to configure a valid provider.");
            std::process::exit(1);
        }
    };
    
    // Test LLM connectivity
    log::info!("🔍 [Server][LLM] Testing LLM connection...");
    let test_prompt = match llm_provider.as_str() {
        "ollama" => {
            // Ollama expects JSON format, so give it a prompt that produces JSON
            "Respond with a JSON object containing a single field 'status' with value 'ok'. Example: {\"status\": \"ok\"}".to_string()
        }
        _ => "Test".to_string()
    };
    let test_dir = std::env::current_dir().unwrap();
    
    match llm_client.query(test_prompt, &test_dir).await {
        Ok(_) => {
            log::info!("✅ [Server][LLM] LLM connection successful");
        }
        Err(e) => {
            log::error!("❌ [Server][LLM] Failed to connect to {}", llm_provider);
            log::error!("");
            
            match llm_provider.as_str() {
                "ollama" => {
                    log::error!("The model might not be downloaded. Pull it with:");
                    log::error!("  ollama pull {}", std::env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.2:latest".to_string()));
                }
                "claude" => {
                    log::error!("Is Claude CLI installed and configured?");
                    log::error!("Check with: claude --version");
                    log::error!("");
                    log::error!("Make sure your API key is set up.");
                }
                _ => {}
            }
            
            log::error!("");
            log::error!("Error details: {}", e);
            std::process::exit(1);
        }
    }
    
    // Initialize prompt system
    let data_dir = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("data");
    log::debug!("Using data directory: {:?}", data_dir);
    let prompt_loader = PromptLoader::new(data_dir);
    let prompt_builder = PromptBuilder::new(prompt_loader);

    // Create shared app state
    let app_state = Arc::new(AppState {
        game_manager,
        llm_client,
        prompt_builder,
    });

    // Build router
    let app = create_router(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    log::info!("🚀 [Server][System] Two Animals server running on http://{addr}");
    
    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_with_timeout())
        .await
        .unwrap();
        
    log::info!("👋 [Server][System] Shut down successfully");
}

async fn shutdown_with_timeout() {
    shutdown_signal().await;
    
    // Start a 5-second timer for graceful shutdown
    log::info!("⏱️  [Shutdown][System] Waiting up to 5 seconds for requests to complete...");
    
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        log::error!("⚠️  [Shutdown][System] Graceful shutdown timeout! Force exiting...");
        std::process::exit(1);
    });
}

async fn shutdown_signal() {
    use tokio::signal;
    
    log::debug!("Installing signal handlers...");
    
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            log::info!("💤 [Shutdown][System] Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            log::info!("💤 [Shutdown][System] Received terminate signal, shutting down gracefully...");
        },
    }
}