use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::Path;
use tokio::process::Command;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn query(&self, prompt: String, working_dir: &Path) -> Result<String>;
}

pub struct ClaudeClient;

#[async_trait]
impl LlmClient for ClaudeClient {
    async fn query(&self, prompt: String, working_dir: &Path) -> Result<String> {
        log::debug!("LLM query from dir: {:?}", working_dir);
        log::debug!("Prompt length: {} chars", prompt.len());
        
        // Add a timeout to LLM queries to prevent hanging
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            Command::new("claude")
                .arg("--print")
                .arg(prompt)
                .current_dir(working_dir)
                .output()
        )
        .await
        .map_err(|_| anyhow!("LLM query timed out after 60 seconds"))??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Claude failed: {}", stderr));
        }

        let response = String::from_utf8_lossy(&output.stdout).to_string();
        
        // Strip markdown code blocks if present
        let cleaned = if response.trim().starts_with("```json") && response.trim().ends_with("```") {
            response
                .trim()
                .strip_prefix("```json")
                .unwrap_or(&response)
                .strip_suffix("```")
                .unwrap_or(&response)
                .trim()
                .to_string()
        } else {
            response
        };
        
        Ok(cleaned)
    }
}

// Future LLM implementations could include:
// - OllamaClient for local models
// - OpenAIClient for GPT models  
// - AnthropicApiClient for Claude API (vs CLI)
// - HuggingFaceClient for open models
// The LlmClient trait makes it easy to swap implementations