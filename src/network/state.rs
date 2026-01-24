use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// The current networking mode
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum NetworkMode {
    #[default]
    Local,          // Single device, no network
    OnlineHost,     // Hosting a multiplayer game
    OnlineClient,   // Joined someone else's game
}

/// Global network state resource
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct NetworkState {
    pub mode: NetworkMode,
    pub room_code: Option<String>,
    pub local_player_id: Option<String>,   // Firebase auth UID
    pub local_player_index: Option<usize>, // Index in Players list
    pub is_connected: bool,
    pub connection_error: Option<String>,
    pub game_state_version: u32,           // Tracks state sync version
}

#[allow(dead_code)]
impl NetworkState {
    pub fn is_host(&self) -> bool {
        matches!(self.mode, NetworkMode::OnlineHost)
    }

    pub fn is_client(&self) -> bool {
        matches!(self.mode, NetworkMode::OnlineClient)
    }

    pub fn is_online(&self) -> bool {
        matches!(self.mode, NetworkMode::OnlineHost | NetworkMode::OnlineClient)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Information about a player in the online lobby
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnlinePlayerInfo {
    pub id: String,
    pub name: String,
    pub character_id: u8,
    pub color_index: usize,
    pub is_ready: bool,
    pub is_connected: bool,
    pub is_host: bool,
}

/// Tracks all players in the current room
#[derive(Resource, Default)]
pub struct RoomPlayers {
    pub players: Vec<OnlinePlayerInfo>,
}

/// Queue for actions received from network (used by host)
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct PendingNetworkActions {
    pub actions: Vec<super::messages::NetworkActionMessage>,
}

/// Latest game state received from network (used by clients)
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct ReceivedGameState {
    pub state_json: Option<String>,
    pub version: u32,
    pub needs_processing: bool,
}
