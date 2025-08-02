use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ExecuteTurnRequest {
    pub repeat: Option<u32>,
    pub endless: bool,
    pub delay_ms: u64,
}

impl Default for ExecuteTurnRequest {
    fn default() -> Self {
        Self {
            repeat: None,
            endless: false,
            delay_ms: default_delay(),
        }
    }
}

fn default_delay() -> u64 {
    1000 // 1 second default delay between turns
}

#[derive(Debug, Serialize)]
pub struct ExecuteTurnResponse {
    pub turns_executed: u32,
    pub last_turn_result: Option<GmResponse>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum Location {
    ForestClearing,
    DeepForest,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameState {
    pub npcs: HashMap<String, Npc>,
    pub contracts: HashMap<String, Contract>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Npc {
    pub name: String,
    pub location: Location,
    pub activity: String,
    pub folder_path: String,
    pub active_contract: Option<String>,
    pub next_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Contract {
    pub id: String,
    pub participants: Vec<String>,
    pub transcript_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub npc: String,
    pub thought: String,
    pub action: String,
    pub dialogue: Option<String>,
}

// Data we send to the GM
#[derive(Debug, Serialize)]
pub struct GmInput {
    pub current_state: CurrentState,
    pub intents: Vec<Intent>,
}

#[derive(Debug, Serialize)]
pub struct CurrentState {
    pub npcs: HashMap<String, Npc>,
    pub active_contracts: HashMap<String, Contract>,
}

// Data we get back from the GM
#[derive(Debug, Serialize, Deserialize)]
pub struct GmResponse {
    pub reality: String,
    pub state_changes: Vec<StateChange>,
    pub contracts: Vec<ContractUpdate>,
    pub next_prompts: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateChange {
    pub npc: String,
    pub location: Location,
    pub activity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractUpdate {
    pub id: String,
    pub participants: Vec<String>,
    pub action: String,  // "create", "update", "end"
    pub transcript_entry: Option<TranscriptEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub reality: String,
    pub details: HashMap<String, NpcAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcAction {
    pub action: String,
    pub dialogue: Option<String>,
}

// Memory update input - what we send to update memories
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryUpdateInput {
    pub npc_name: String,
    pub intent: Intent,
    pub reality: String,
    pub other_npcs_present: Vec<String>,
}
