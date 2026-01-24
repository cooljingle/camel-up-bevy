pub mod state;
pub mod messages;
pub mod room;

#[cfg(target_arch = "wasm32")]
pub mod js_bindings;

#[cfg(target_arch = "wasm32")]
pub mod sync;

use bevy::prelude::*;
use state::{NetworkState, NetworkMode, RoomPlayers, PendingNetworkActions, ReceivedGameState};

/// Plugin that handles all multiplayer networking functionality
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkState>()
            .init_resource::<RoomPlayers>()
            .init_resource::<PendingNetworkActions>()
            .init_resource::<ReceivedGameState>();

        // Add WASM-specific systems
        #[cfg(target_arch = "wasm32")]
        {
            app.add_systems(Update, (
                sync::poll_firebase_updates,
                sync::process_received_game_state,
                sync::broadcast_game_state_system,
            ).run_if(resource_exists::<crate::components::Players>));
        }
    }
}

/// Check if we're in online mode
#[allow(dead_code)]
pub fn is_online(network_state: &NetworkState) -> bool {
    matches!(network_state.mode, NetworkMode::OnlineHost | NetworkMode::OnlineClient)
}

/// Check if local player can take actions (their turn in online mode, or always in local mode)
#[allow(dead_code)]
pub fn can_local_player_act(network_state: &NetworkState, current_player_index: usize, players: &crate::components::Players) -> bool {
    match network_state.mode {
        NetworkMode::Local => true,
        NetworkMode::OnlineHost | NetworkMode::OnlineClient => {
            // Check if the current player is the local player
            if let Some(local_player_index) = network_state.local_player_index {
                current_player_index == local_player_index && !players.current_player().is_ai
            } else {
                false
            }
        }
    }
}
