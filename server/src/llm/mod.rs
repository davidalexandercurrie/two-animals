pub mod client;
pub mod ollama;
pub mod parser;

pub use client::{ClaudeClient, LlmClient};
pub use ollama::OllamaClient;