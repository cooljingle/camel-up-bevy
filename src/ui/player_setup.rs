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
    pub color_index: usize,
    /// Tracks if the user manually edited the name (prevents auto-name updates)
    pub name_edited: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_ai: false,
            character_id: CharacterId::default(),
            color_index: 0,
            name_edited: false,
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
        // Start with 1 human + 3 AI players by default
        // Use sequential character and color indices (0, 1, 2, 3)
        // Player 1: Scholar (0), Red (0)
        // Player 2: Merchant (1), Blue (1)
        // Player 3: Princess (2), Green (2)
        // Player 4: Jockey (3), Yellow (3)
        Self {
            players: vec![
                PlayerConfig {
                    name: "Player 1".to_string(),
                    is_ai: false,
                    character_id: CharacterId::from_index(0), // Scholar
                    color_index: 0, // Red
                    name_edited: false,
                },
                PlayerConfig {
                    name: CharacterId::from_index(1).random_name(),
                    is_ai: true,
                    character_id: CharacterId::from_index(1), // Merchant
                    color_index: 1, // Blue
                    name_edited: false,
                },
                PlayerConfig {
                    name: CharacterId::from_index(2).random_name(),
                    is_ai: true,
                    character_id: CharacterId::from_index(2), // Princess
                    color_index: 2, // Green
                    name_edited: false,
                },
                PlayerConfig {
                    name: CharacterId::from_index(3).random_name(),
                    is_ai: true,
                    character_id: CharacterId::from_index(3), // Jockey
                    color_index: 3, // Yellow
                    name_edited: false,
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
            // Find an unused character
            let used: HashSet<CharacterId> = self.players.iter().map(|p| p.character_id).collect();
            let available = (0..16)
                .map(CharacterId::from_index)
                .find(|c| !used.contains(c))
                .unwrap_or(CharacterId::default());

            // Find an unused color index
            let used_colors: HashSet<usize> = self.players.iter().map(|p| p.color_index).collect();
            let available_color = (0..8)
                .find(|c| !used_colors.contains(c))
                .unwrap_or(0);

            // New players default to AI with avatar-based name
            self.players.push(PlayerConfig {
                name: available.random_name(),
                is_ai: true,
                character_id: available,
                color_index: available_color,
                name_edited: false,
            });
        }
    }

    pub fn remove_player(&mut self) {
        if self.players.len() > Self::MIN_PLAYERS {
            self.players.pop();
        }
    }

    /// Randomize color, avatar, and name for all players
    pub fn randomize_players(&mut self) {
        let mut rng = rand::thread_rng();

        // Create shuffled character IDs
        let mut character_ids: Vec<CharacterId> = (0..16)
            .map(CharacterId::from_index)
            .collect();
        character_ids.shuffle(&mut rng);

        // Create shuffled color indices
        let mut color_indices: Vec<usize> = (0..8).collect();
        color_indices.shuffle(&mut rng);

        // Assign to each player
        for (i, player) in self.players.iter_mut().enumerate() {
            player.character_id = character_ids[i % character_ids.len()];
            player.color_index = color_indices[i % color_indices.len()];

            // Update name based on new character (if not manually edited)
            if !player.name_edited {
                if player.is_ai {
                    player.name = player.character_id.random_name();
                } else {
                    player.name = format!("Player {}", i + 1);
                }
            }
        }
    }

    /// Update name when toggling between Human and AI (if name wasn't manually edited)
    pub fn set_player_is_ai(&mut self, player_index: usize, is_ai: bool) {
        if player_index >= self.players.len() {
            return;
        }
        let player = &mut self.players[player_index];
        if player.is_ai == is_ai {
            return; // No change
        }
        player.is_ai = is_ai;
        // Only update name if it wasn't manually edited
        if !player.name_edited {
            if is_ai {
                player.name = player.character_id.random_name();
            } else {
                player.name = format!("Player {}", player_index + 1);
            }
        }
    }

    /// Convert to the format expected by Players::new()
    /// If randomize_start_order is true, shuffles the player order
    pub fn to_player_configs(&self) -> Vec<(String, bool, CharacterId, usize)> {
        let mut configs: Vec<(String, bool, CharacterId, usize)> = self.players
            .iter()
            .map(|p| (p.name.clone(), p.is_ai, p.character_id, p.color_index))
            .collect();

        if self.randomize_start_order {
            let mut rng = rand::thread_rng();
            configs.shuffle(&mut rng);
        }

        configs
    }
}
