use serde::{Deserialize, Serialize};

/// Actions that can be sent over the network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkAction {
    RollPyramid,
    TakeLegBet { color: String },
    PlaceSpectatorTile { space_index: u8, is_oasis: bool },
    PlaceRaceBet { color: String, is_winner_bet: bool },
}

/// A network action with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkActionMessage {
    pub player_id: String,
    pub action: NetworkAction,
    pub timestamp: u64,
}

/// Serializable version of camel position
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableCamelPosition {
    pub color: String,
    pub space_index: u8,
    pub stack_position: u8,
}

/// Serializable version of a player
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializablePlayer {
    pub id: u8,
    pub network_id: String,
    pub name: String,
    pub money: i32,
    pub has_spectator_tile: bool,
    pub available_race_cards: Vec<String>,
    pub is_ai: bool,
    pub character_id: u8,
    pub color_index: usize,
}

/// Serializable turn state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableTurnState {
    pub current_player: usize,
    pub action_taken: bool,
    pub leg_number: u32,
    pub awaiting_action: bool,
    pub leg_has_started: bool,
}

/// Serializable pyramid state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializablePyramid {
    pub rolled_dice: Vec<SerializableDieResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableDieResult {
    pub color: String,
    pub value: u8,
    pub is_crazy: bool,
}

/// Serializable leg betting tiles state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableLegBettingTiles {
    pub tiles: Vec<(String, Vec<u8>)>, // (color, available values)
}

/// Serializable race bet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableRaceBet {
    pub camel_color: String,
    pub player_id: u8,
}

/// Serializable spectator tile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableSpectatorTile {
    pub space_index: u8,
    pub owner_id: u8,
    pub is_oasis: bool,
}

/// Serializable leg bet for a player
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableLegBet {
    pub camel_color: String,
    pub value: u8,
}

/// Complete game state for network sync
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableGameState {
    pub version: u32,
    pub turn_state: SerializableTurnState,
    pub players: Vec<SerializablePlayer>,
    pub camels: Vec<SerializableCamelPosition>,
    pub crazy_camels: Vec<SerializableCamelPosition>,
    pub pyramid: SerializablePyramid,
    pub leg_betting_tiles: SerializableLegBettingTiles,
    pub winner_bets: Vec<SerializableRaceBet>,
    pub loser_bets: Vec<SerializableRaceBet>,
    pub placed_spectator_tiles: Vec<SerializableSpectatorTile>,
    pub player_leg_bets: Vec<Vec<SerializableLegBet>>,
    pub player_pyramid_tokens: Vec<u8>,
}

/// Room metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomMetadata {
    pub host_id: String,
    pub created_at: u64,
    pub game_started: bool,
    pub max_players: u8,
}

/// Player info stored in Firebase
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FirebasePlayerInfo {
    pub name: String,
    pub character_id: u8,
    pub color_index: usize,
    pub is_ready: bool,
    pub is_connected: bool,
}
