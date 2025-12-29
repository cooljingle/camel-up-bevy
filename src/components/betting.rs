use bevy::prelude::*;
use super::CamelColor;

#[derive(Clone, Debug)]
pub struct LegBetTile {
    pub camel: CamelColor,
    pub value: u8, // 5, 3, or 2
}

#[derive(Resource)]
pub struct LegBettingTiles {
    pub stacks: Vec<Vec<LegBetTile>>, // One stack per camel color
}

impl LegBettingTiles {
    pub fn new() -> Self {
        let stacks = CamelColor::all()
            .into_iter()
            .map(|color| {
                vec![
                    LegBetTile { camel: color, value: 2 }, // Bottom
                    LegBetTile { camel: color, value: 3 },
                    LegBetTile { camel: color, value: 5 }, // Top
                ]
            })
            .collect();

        Self { stacks }
    }

    pub fn take_tile(&mut self, color: CamelColor) -> Option<LegBetTile> {
        let stack_index = CamelColor::all()
            .iter()
            .position(|&c| c == color)?;

        self.stacks[stack_index].pop()
    }

    pub fn top_tile(&self, color: CamelColor) -> Option<&LegBetTile> {
        let stack_index = CamelColor::all()
            .iter()
            .position(|&c| c == color)?;

        self.stacks[stack_index].last()
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for LegBettingTiles {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct RaceBet {
    pub camel: CamelColor,
    pub player_id: u8,
}

#[derive(Resource, Default)]
pub struct RaceBets {
    pub winner_bets: Vec<RaceBet>,
    pub loser_bets: Vec<RaceBet>,
}

impl RaceBets {
    pub fn place_winner_bet(&mut self, camel: CamelColor, player_id: u8) {
        self.winner_bets.push(RaceBet { camel, player_id });
    }

    pub fn place_loser_bet(&mut self, camel: CamelColor, player_id: u8) {
        self.loser_bets.push(RaceBet { camel, player_id });
    }
}

#[derive(Component)]
pub struct PlayerLegBets {
    pub tiles: Vec<LegBetTile>,
}

impl Default for PlayerLegBets {
    fn default() -> Self {
        Self { tiles: Vec::new() }
    }
}

#[derive(Component)]
pub struct PyramidTokenHolder {
    pub count: u8,
}

impl Default for PyramidTokenHolder {
    fn default() -> Self {
        Self { count: 0 }
    }
}
