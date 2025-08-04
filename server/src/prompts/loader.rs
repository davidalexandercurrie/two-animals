use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct PromptLoader {
    data_dir: PathBuf,
}

impl PromptLoader {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn load_npc_base(&self) -> Result<String> {
        let path = self.data_dir.join("prompts/core/npc_base.md");
        fs::read_to_string(&path)
            .with_context(|| format!("Failed to load NPC base prompt from {:?}", path))
    }

    pub fn load_personality(&self, npc_name: &str) -> Result<String> {
        let path = self.data_dir.join(format!("npcs/{}/personality.md", npc_name));
        fs::read_to_string(&path)
            .with_context(|| format!("Failed to load personality for {} from {:?}", npc_name, path))
    }

    pub fn load_gm_base(&self) -> Result<String> {
        let path = self.data_dir.join("prompts/gm/gm_base.md");
        fs::read_to_string(&path)
            .with_context(|| format!("Failed to load GM base prompt from {:?}", path))
    }

    pub fn load_memories(&self, npc_name: &str) -> Result<String> {
        let memory_path = self.data_dir.join(format!("npcs/{}/memories.json", npc_name));
        
        if memory_path.exists() {
            fs::read_to_string(&memory_path)
                .with_context(|| format!("Failed to load memories for {} from {:?}", npc_name, memory_path))
        } else {
            // Try initial_memories.json as fallback
            let initial_path = self.data_dir.join(format!("npcs/{}/initial_memories.json", npc_name));
            if initial_path.exists() {
                fs::read_to_string(&initial_path)
                    .with_context(|| format!("Failed to load initial memories for {} from {:?}", npc_name, initial_path))
            } else {
                // Return empty memory structure
                Ok(r#"{
                    "self_memories": {
                        "immediate_context": "",
                        "recent_events": [],
                        "core_memories": []
                    },
                    "relationships": {}
                }"#.to_string())
            }
        }
    }
}