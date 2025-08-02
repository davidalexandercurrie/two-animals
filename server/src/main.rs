use std::sync::Arc;
use server::{
    AppState, ClaudeClient, GameStateManager, LlmClient, PromptBuilder, PromptLoader,
    create_router, logging,
};

#[tokio::main]
async fn main() {
    // Initialize logger
    logging::init_logger();
    
    let addr = "0.0.0.0:3000";

    // Initialize game state
    let game_manager = GameStateManager::new();
    
    // Initialize LLM client
    let llm_client: Arc<dyn LlmClient> = Arc::new(ClaudeClient);
    
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