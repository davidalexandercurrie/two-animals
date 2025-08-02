use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySystem {
    pub self_memories: SelfMemories,
    pub relationships: HashMap<String, RelationshipMemory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfMemories {
    pub immediate_context: String,
    pub recent_events: Vec<String>,
    pub core_memories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMemory {
    pub immediate_context: String,
    pub recent_memories: Vec<Memory>,
    pub long_term_summary: String,
    pub core_memories: Vec<String>,
    pub current_sentiment: f32,  // -1.0 to 1.0
    pub overall_bond: f32,       // -1.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub event: String,
    pub timestamp: DateTime<Utc>,
    pub emotional_impact: String,
    pub importance: f32,  // 0.0 to 1.0
}

// Input from LLM when updating memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUpdate {
    pub immediate_self_context: String,
    pub new_self_memory: Option<String>,
    pub relationship_updates: HashMap<String, RelationshipUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipUpdate {
    pub immediate_context: String,
    pub new_memory: Option<Memory>,
    pub current_sentiment: f32,
    pub long_term_summary_update: Option<String>,
    pub potential_core_memory: Option<String>,
}

// Used when a memory needs to fade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FadeDecision {
    pub memory_to_fade: Memory,
    pub impacts_long_term: bool,
    pub new_long_term_summary: Option<String>,
    pub forms_core_memory: bool,
}