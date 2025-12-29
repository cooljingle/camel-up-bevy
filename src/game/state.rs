use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    PlayerSetup,
    Playing,
    LegScoring,
    GameEnd,
}
