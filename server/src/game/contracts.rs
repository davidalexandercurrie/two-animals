use crate::types::{Contract, TranscriptEntry};
use anyhow::Result;

pub struct ContractManager;

impl ContractManager {
    pub fn create_contract(
        participants: Vec<String>,
        initial_entry: Option<TranscriptEntry>,
    ) -> Result<Contract> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f").to_string();
        let contract_id = format!("conv_{timestamp}");
        
        let contract = Contract {
            id: contract_id.clone(),
            participants,
            transcript_file: format!("../data/contracts/{contract_id}.json"),
        };
        
        // Create contract file with initial entry if provided
        if let Some(entry) = initial_entry {
            let transcript = vec![entry];
            let json = serde_json::to_string_pretty(&transcript)?;
            
            // Create contracts directory if it doesn't exist
            std::fs::create_dir_all("../data/contracts").ok();
            
            std::fs::write(&contract.transcript_file, json)?;
        }
        
        Ok(contract)
    }
    
    pub fn update_contract(
        contract: &Contract,
        entry: TranscriptEntry,
    ) -> Result<()> {
        // Read existing transcript
        let contents = std::fs::read_to_string(&contract.transcript_file)
            .unwrap_or_else(|_| "[]".to_string());
        let mut transcript: Vec<TranscriptEntry> =
            serde_json::from_str(&contents).unwrap_or_else(|_| vec![]);
        
        // Append new entry
        transcript.push(entry);
        
        // Write back
        let json = serde_json::to_string_pretty(&transcript)?;
        std::fs::write(&contract.transcript_file, json)?;
        
        Ok(())
    }
    
    pub fn read_contract_transcript(contract_id: &str) -> Result<Vec<TranscriptEntry>> {
        let contract_path = format!("../data/contracts/{contract_id}.json");
        let contents = std::fs::read_to_string(&contract_path)?;
        let transcript: Vec<TranscriptEntry> = serde_json::from_str(&contents)?;
        Ok(transcript)
    }
}