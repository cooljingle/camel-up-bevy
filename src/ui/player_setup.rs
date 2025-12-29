use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::state::GameState;
use crate::game::ai::{AiConfig, AiDifficulty};
use rand::seq::SliceRandom;

/// Configuration for a single player during setup
#[derive(Clone)]
pub struct PlayerConfig {
    pub name: String,
    pub is_ai: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_ai: false,
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
        Self {
            players: vec![
                PlayerConfig { name: "Player 1".to_string(), is_ai: false },
                PlayerConfig { name: "Player 2".to_string(), is_ai: true },
                PlayerConfig { name: "Player 3".to_string(), is_ai: true },
                PlayerConfig { name: "Player 4".to_string(), is_ai: true },
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
            self.players.push(PlayerConfig {
                name: format!("Player {}", num),
                is_ai: false,
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
    pub fn to_player_configs(&self) -> Vec<(String, bool)> {
        let mut configs: Vec<(String, bool)> = self.players
            .iter()
            .map(|p| (p.name.clone(), p.is_ai))
            .collect();

        if self.randomize_start_order {
            let mut rng = rand::thread_rng();
            configs.shuffle(&mut rng);
        }

        configs
    }
}

pub fn player_setup_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut config: ResMut<PlayerSetupConfig>,
    mut ai_config: ResMut<AiConfig>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);

            ui.heading(egui::RichText::new("Player Setup").size(36.0));
            ui.add_space(20.0);

            // Player count controls
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Players: {}", config.players.len())).size(20.0));
                ui.add_space(20.0);

                let can_remove = config.players.len() > PlayerSetupConfig::MIN_PLAYERS;
                let can_add = config.players.len() < PlayerSetupConfig::MAX_PLAYERS;

                if ui.add_enabled(can_remove, egui::Button::new("-").min_size(egui::vec2(30.0, 30.0))).clicked() {
                    config.remove_player();
                }
                if ui.add_enabled(can_add, egui::Button::new("+").min_size(egui::vec2(30.0, 30.0))).clicked() {
                    config.add_player();
                }
            });

            ui.add_space(20.0);

            // Scrollable player list
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for i in 0..config.players.len() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{}.", i + 1)).size(18.0));
                                ui.add_space(10.0);

                                // Name input
                                ui.label("Name:");
                                let name = &mut config.players[i].name;
                                ui.add(egui::TextEdit::singleline(name).desired_width(150.0));

                                ui.add_space(20.0);

                                // Human/AI toggle
                                let is_ai = &mut config.players[i].is_ai;
                                ui.selectable_value(is_ai, false, "Human");
                                ui.selectable_value(is_ai, true, "AI");
                            });
                        });
                        ui.add_space(5.0);
                    }
                });

            ui.add_space(20.0);

            // AI Difficulty selector (only show if there are AI players)
            let has_ai_players = config.players.iter().any(|p| p.is_ai);
            if has_ai_players {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("AI Difficulty:").size(18.0));
                        ui.add_space(10.0);

                        egui::ComboBox::from_id_salt("ai_difficulty")
                            .selected_text(match ai_config.difficulty {
                                AiDifficulty::Random => "Random",
                                AiDifficulty::Basic => "Basic",
                                AiDifficulty::Smart => "Smart",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut ai_config.difficulty, AiDifficulty::Random, "Random - Picks randomly");
                                ui.selectable_value(&mut ai_config.difficulty, AiDifficulty::Basic, "Basic - Simple heuristics");
                                ui.selectable_value(&mut ai_config.difficulty, AiDifficulty::Smart, "Smart - Probability-based");
                            });
                    });
                });
            }

            ui.add_space(30.0);

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("Back").size(20.0)).clicked() {
                    next_state.set(GameState::MainMenu);
                }

                ui.add_space(40.0);

                if ui.button(egui::RichText::new("Start Game").size(24.0)).clicked() {
                    next_state.set(GameState::Playing);
                }
            });
        });
    });
}
