use crate::types::{Contract, GameState, Location, Npc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct GameStateManager {
    pub state: Arc<Mutex<GameState>>,
}

impl GameStateManager {
    pub fn new() -> Self {
        let mut npcs = HashMap::new();
        
        npcs.insert(
            "bear".to_string(),
            Npc {
                name: "bear".to_string(),
                location: Location::ForestClearing,
                activity: "resting".to_string(),
                folder_path: "../data/npcs/bear".to_string(),
                active_contract: None,
                next_prompt: None,
            },
        );
        
        npcs.insert(
            "wolf".to_string(),
            Npc {
                name: "wolf".to_string(),
                location: Location::ForestClearing,
                activity: "patrolling".to_string(),
                folder_path: "../data/npcs/wolf".to_string(),
                active_contract: None,
                next_prompt: None,
            },
        );
        
        let contracts = HashMap::new();
        let game_state = GameState { npcs, contracts };
        
        Self {
            state: Arc::new(Mutex::new(game_state)),
        }
    }
    
    pub fn get_state(&self) -> GameState {
        self.state.lock().unwrap().clone()
    }
    
    pub fn update_npc_location(&self, npc_name: &str, location: Location, activity: String) {
        let mut game = self.state.lock().unwrap();
        if let Some(npc) = game.npcs.get_mut(npc_name) {
            npc.location = location;
            npc.activity = activity;
        }
    }
    
    pub fn set_npc_contract(&self, npc_name: &str, contract_id: Option<String>) {
        let mut game = self.state.lock().unwrap();
        if let Some(npc) = game.npcs.get_mut(npc_name) {
            npc.active_contract = contract_id;
        }
    }
    
    pub fn set_npc_prompt(&self, npc_name: &str, prompt: String) {
        let mut game = self.state.lock().unwrap();
        if let Some(npc) = game.npcs.get_mut(npc_name) {
            npc.next_prompt = Some(prompt);
        }
    }
    
    pub fn add_contract(&self, contract: Contract) {
        let mut game = self.state.lock().unwrap();
        game.contracts.insert(contract.id.clone(), contract);
    }
    
    pub fn get_contract(&self, contract_id: &str) -> Option<Contract> {
        let game = self.state.lock().unwrap();
        game.contracts.get(contract_id).cloned()
    }
}