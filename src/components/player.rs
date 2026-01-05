use bevy::prelude::*;
use std::collections::HashSet;
use super::CamelColor;
use crate::ui::characters::CharacterId;

#[derive(Resource)]
pub struct Players {
    pub players: Vec<PlayerData>,
    pub current_player_index: usize,
}

#[derive(Clone)]
pub struct PlayerData {
    pub id: u8,
    pub name: String,
    pub money: i32,
    pub has_spectator_tile: bool,
    pub available_race_cards: HashSet<CamelColor>,
    pub is_ai: bool,
    pub character_id: CharacterId,
    pub color_index: usize,
}

impl PlayerData {
    pub fn new(id: u8, name: String, is_ai: bool) -> Self {
        Self {
            id,
            name,
            money: 3, // Starting money
            has_spectator_tile: true,
            available_race_cards: CamelColor::all().into_iter().collect(),
            is_ai,
            character_id: CharacterId::from_index(id as usize),
            color_index: id as usize,
        }
    }
}

impl Players {
    pub fn new(player_configs: Vec<(String, bool, CharacterId, usize)>) -> Self {
        let players = player_configs
            .into_iter()
            .enumerate()
            .map(|(i, (name, is_ai, character_id, color_index))| {
                let mut player = PlayerData::new(i as u8, name, is_ai);
                player.character_id = character_id;
                player.color_index = color_index;
                player
            })
            .collect();

        Self {
            players,
            current_player_index: 0,
        }
    }

    pub fn current_player(&self) -> &PlayerData {
        &self.players[self.current_player_index]
    }

    pub fn current_player_mut(&mut self) -> &mut PlayerData {
        &mut self.players[self.current_player_index]
    }

    pub fn advance_turn(&mut self) {
        self.current_player_index = (self.current_player_index + 1) % self.players.len();
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}

impl Default for Players {
    fn default() -> Self {
        // Default 2-player game for testing
        Self::new(vec![
            ("Player 1".to_string(), false, CharacterId::from_index(0), 0),
            ("Player 2 (AI)".to_string(), true, CharacterId::from_index(1), 1),
        ])
    }
}
