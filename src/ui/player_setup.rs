use bevy::prelude::*;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use crate::ui::characters::CharacterId;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Check if running on a mobile device (WASM only)
/// Reads from window.isMobileDevice set by JavaScript
#[cfg(target_arch = "wasm32")]
pub fn is_mobile_device() -> bool {
    use web_sys::window;

    if let Some(win) = window() {
        // Try to read window.isMobileDevice
        if let Ok(value) = js_sys::Reflect::get(&win, &JsValue::from_str("isMobileDevice")) {
            return value.as_bool().unwrap_or(false);
        }
    }
    false
}

/// Check if running on iPhone (WASM only)
/// Fullscreen API is not supported on iPhone
#[cfg(target_arch = "wasm32")]
pub fn is_iphone() -> bool {
    use web_sys::window;

    if let Some(win) = window() {
        if let Ok(value) = js_sys::Reflect::get(&win, &JsValue::from_str("isIPhone")) {
            return value.as_bool().unwrap_or(false);
        }
    }
    false
}

/// Non-WASM stub - always returns false
#[cfg(not(target_arch = "wasm32"))]
pub fn is_iphone() -> bool {
    false
}

/// Configuration for a single player during setup
#[derive(Clone)]
pub struct PlayerConfig {
    pub name: String,
    pub is_ai: bool,
    pub character_id: CharacterId,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_ai: false,
            character_id: CharacterId::default(),
        }
    }
}

/// Resource to hold player configuration state during setup
#[derive(Resource)]
pub struct PlayerSetupConfig {
    pub players: Vec<PlayerConfig>,
    pub randomize_start_order: bool,
}

impl Default for PlayerSetupConfig {
    fn default() -> Self {
        // Create shuffled character IDs so each game starts with random characters
        let mut rng = rand::thread_rng();
        let mut character_ids: Vec<CharacterId> = (0..8)
            .map(CharacterId::from_index)
            .collect();
        character_ids.shuffle(&mut rng);

        // Start with 1 human + 3 AI players by default
        // AI players get random thematic names based on their character
        Self {
            players: vec![
                PlayerConfig {
                    name: "Player 1".to_string(),
                    is_ai: false,
                    character_id: character_ids[0]
                },
                PlayerConfig {
                    name: character_ids[1].random_name(),
                    is_ai: true,
                    character_id: character_ids[1]
                },
                PlayerConfig {
                    name: character_ids[2].random_name(),
                    is_ai: true,
                    character_id: character_ids[2]
                },
                PlayerConfig {
                    name: character_ids[3].random_name(),
                    is_ai: true,
                    character_id: character_ids[3]
                },
            ],
            randomize_start_order: false,
        }
    }
}

impl PlayerSetupConfig {
    pub const MIN_PLAYERS: usize = 2;
    pub const MAX_PLAYERS: usize = 8;

    pub fn add_player(&mut self) {
        if self.players.len() < Self::MAX_PLAYERS {
            let num = self.players.len() + 1;
            // Find an unused character
            let used: HashSet<CharacterId> = self.players.iter().map(|p| p.character_id).collect();
            let available = (0..8)
                .map(CharacterId::from_index)
                .find(|c| !used.contains(c))
                .unwrap_or(CharacterId::default());

            self.players.push(PlayerConfig {
                name: format!("Player {}", num),
                is_ai: false,
                character_id: available,
            });
        }
    }

    pub fn remove_player(&mut self) {
        if self.players.len() > Self::MIN_PLAYERS {
            self.players.pop();
        }
    }

    /// Convert to the format expected by Players::new()
    /// If randomize_start_order is true, shuffles the player order
    pub fn to_player_configs(&self) -> Vec<(String, bool, CharacterId)> {
        let mut configs: Vec<(String, bool, CharacterId)> = self.players
            .iter()
            .map(|p| (p.name.clone(), p.is_ai, p.character_id))
            .collect();

        if self.randomize_start_order {
            let mut rng = rand::thread_rng();
            configs.shuffle(&mut rng);
        }

        configs
    }
}
