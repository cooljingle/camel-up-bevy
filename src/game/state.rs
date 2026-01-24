use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Lobby,        // Create/join online game room
    WaitingRoom,  // Waiting for players before game starts
    Playing,
    GameEnd,
}
