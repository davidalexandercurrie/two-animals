use anyhow::Result;
use std::path::Path;

pub struct NpcRegistry;

impl NpcRegistry {
    pub fn new() -> Self {
        Self
    }
    
    // TODO: Implement loading NPCs from data directory
    pub fn load_from_directory(_data_dir: &Path) -> Result<Self> {
        Ok(Self::new())
    }
}