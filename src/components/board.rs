use bevy::prelude::*;
use std::collections::HashMap;

pub const TRACK_LENGTH: u8 = 16;
pub const FINISH_LINE: u8 = 16;

#[derive(Component)]
pub struct BoardSpace {
    pub index: u8,
}

#[derive(Component, Clone, Copy)]
pub struct DesertTile {
    pub owner_id: u8,
    pub is_oasis: bool, // true = oasis (+1 forward, on top), false = mirage (-1 backward, underneath)
}

/// Resource to track all placed desert tiles on the board
#[derive(Resource, Default)]
pub struct PlacedDesertTiles {
    /// Map of space_index -> (owner_id, is_oasis)
    pub tiles: HashMap<u8, (u8, bool)>,
}

impl PlacedDesertTiles {
    pub fn place_tile(&mut self, space_index: u8, owner_id: u8, is_oasis: bool) {
        self.tiles.insert(space_index, (owner_id, is_oasis));
    }

    pub fn remove_player_tile(&mut self, owner_id: u8) -> Option<u8> {
        let mut found_space = None;
        for (&space, &(owner, _)) in &self.tiles {
            if owner == owner_id {
                found_space = Some(space);
                break;
            }
        }
        if let Some(space) = found_space {
            self.tiles.remove(&space);
        }
        found_space
    }

    pub fn get_tile(&self, space_index: u8) -> Option<(u8, bool)> {
        self.tiles.get(&space_index).copied()
    }

    pub fn is_space_occupied(&self, space_index: u8) -> bool {
        self.tiles.contains_key(&space_index)
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
    }
}

#[derive(Component)]
pub struct Track;

#[derive(Resource)]
pub struct GameBoard {
    pub space_positions: Vec<Vec2>, // World positions for each track space
}

impl GameBoard {
    pub fn new() -> Self {
        // Create an oval track layout
        // Spaces 0-7 on bottom row (left to right)
        // Spaces 8-15 on top row (right to left)
        let mut positions = Vec::with_capacity(TRACK_LENGTH as usize);

        let spacing = 80.0;
        let row_height = 200.0;
        let start_x = -280.0;

        // Bottom row (0-7): left to right
        for i in 0..8 {
            positions.push(Vec2::new(start_x + (i as f32 * spacing), -row_height / 2.0));
        }

        // Top row (8-15): right to left
        for i in 0..8 {
            positions.push(Vec2::new(start_x + ((7 - i) as f32 * spacing), row_height / 2.0));
        }

        Self {
            space_positions: positions,
        }
    }

    pub fn get_position(&self, space_index: u8) -> Vec2 {
        self.space_positions
            .get(space_index as usize)
            .copied()
            .unwrap_or(Vec2::ZERO)
    }
}

impl Default for GameBoard {
    fn default() -> Self {
        Self::new()
    }
}
