use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rand::Rng;
use crate::components::{Players, PlacedDesertTiles, DesertTile, CamelColor, Camel, BoardPosition, RaceBets};
use crate::game::state::GameState;
use crate::systems::movement::{get_leading_camel, get_second_place_camel, get_last_place_camel};
use crate::systems::turn::{PlayerLegBetsStore, PlayerPyramidTokens};
use crate::systems::animation::{spawn_firework, random_firework_color};
use crate::ui::characters::{draw_avatar, draw_avatar_with_expression};
use crate::ui::hud::{camel_color_to_egui, draw_camel_silhouette, draw_mini_leg_bet_card};

/// Player colors for visual distinction (same as in hud.rs)
const PLAYER_COLORS: [egui::Color32; 8] = [
    egui::Color32::from_rgb(220, 50, 50),   // Red
    egui::Color32::from_rgb(50, 120, 220),  // Blue
    egui::Color32::from_rgb(50, 180, 80),   // Green
    egui::Color32::from_rgb(220, 180, 50),  // Yellow
    egui::Color32::from_rgb(180, 80, 220),  // Purple
    egui::Color32::from_rgb(220, 130, 50),  // Orange
    egui::Color32::from_rgb(80, 200, 200),  // Cyan
    egui::Color32::from_rgb(200, 100, 150), // Pink
];

/// Phase of the game end sequence
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GameEndPhase {
    #[default]
    LegComplete,          // Show final leg scoring (same as normal leg)
    StandingsPreBets,     // Show standings before long-term bets
    RevealingWinnerBets,  // Animate winner bets one by one
    RevealingLoserBets,   // Animate loser bets one by one
    FinalResults,         // Show final winner
}

/// Data for a pending bet reveal
#[derive(Clone, Debug)]
pub struct PendingBetReveal {
    pub player_id: u8,
    pub player_name: String,
    pub player_character_id: crate::ui::characters::CharacterId,
    pub camel: CamelColor,
    pub is_correct: bool,
    pub payout: i32,          // Positive for correct, -1 for wrong
    pub bet_order: usize,     // Position in the reveal sequence (for payout calculation)
}

/// State for the game end sequence
#[derive(Resource, Default)]
pub struct GameEndState {
    pub phase: GameEndPhase,
    pub leg_scores_applied: bool,
    pub winner_bets_to_reveal: Vec<PendingBetReveal>,
    pub loser_bets_to_reveal: Vec<PendingBetReveal>,
    pub current_reveal_index: usize,
    pub reveal_timer: f32,
    pub reveal_animation_duration: f32,
    pub winning_camel: Option<CamelColor>,
    pub losing_camel: Option<CamelColor>,
    // Track scores before long-term bets are applied
    pub scores_before_long_term: Vec<(String, i32, crate::ui::characters::CharacterId, u8)>, // (name, money, character_id, player_id)
}

impl GameEndState {
    pub fn new() -> Self {
        Self {
            phase: GameEndPhase::LegComplete,
            leg_scores_applied: false,
            winner_bets_to_reveal: Vec::new(),
            loser_bets_to_reveal: Vec::new(),
            current_reveal_index: 0,
            reveal_timer: 0.0,
            reveal_animation_duration: 1.5, // Time per bet reveal
            winning_camel: None,
            losing_camel: None,
            scores_before_long_term: Vec::new(),
        }
    }
}

/// State for firework celebration on game end
#[derive(Resource, Default)]
pub struct CelebrationState {
    pub active: bool,
    pub elapsed: f32,
    pub next_firework_time: f32,
    pub duration: f32,  // How long the celebration lasts
}

pub fn leg_scoring_ui(
    mut contexts: EguiContexts,
    players: Option<ResMut<Players>>,
    mut next_state: ResMut<NextState<GameState>>,
    pyramid: Option<ResMut<crate::components::Pyramid>>,
    leg_tiles: Option<ResMut<crate::components::LegBettingTiles>>,
    player_leg_bets: Option<ResMut<PlayerLegBetsStore>>,
    player_pyramid_tokens: Option<ResMut<PlayerPyramidTokens>>,
    turn_state: Option<ResMut<crate::systems::turn::TurnState>>,
    placed_tiles: Option<ResMut<PlacedDesertTiles>>,
    desert_tile_entities: Query<Entity, With<DesertTile>>,
    mut commands: Commands,
) {
    let Some(mut players) = players else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Collect player data for display before any mutable operations
    let mut sorted_players: Vec<_> = players.players.iter()
        .map(|p| (p.name.clone(), p.money))
        .collect();
    sorted_players.sort_by(|a, b| b.1.cmp(&a.1));

    let mut should_continue = false;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(egui::RichText::new("Leg Complete!").size(36.0));
            ui.add_space(30.0);

            ui.heading("Current Standings");
            ui.add_space(10.0);

            for (rank, (name, money)) in sorted_players.iter().enumerate() {
                let rank_text = match rank {
                    0 => "1st",
                    1 => "2nd",
                    2 => "3rd",
                    3 => "4th",
                    4 => "5th",
                    5 => "6th",
                    6 => "7th",
                    7 => "8th",
                    _ => "   ",
                };
                ui.label(format!("{}: {} - ${}", rank_text, name, money));
            }

            ui.add_space(40.0);

            if ui.button(egui::RichText::new("Continue to Next Leg").size(20.0)).clicked() {
                should_continue = true;
            }
        });
    });

    if should_continue {
        // Reset for new leg
        if let Some(mut pyramid) = pyramid {
            pyramid.reset();
        }
        if let Some(mut leg_tiles) = leg_tiles {
            leg_tiles.reset();
        }
        if let Some(mut player_leg_bets) = player_leg_bets {
            player_leg_bets.clear_all();
        }
        if let Some(mut player_pyramid_tokens) = player_pyramid_tokens {
            player_pyramid_tokens.clear_all();
        }
        if let Some(mut turn_state) = turn_state {
            turn_state.leg_number += 1;
            turn_state.action_taken = false;
            turn_state.awaiting_action = true;
            turn_state.leg_has_started = false; // Reset for new leg
            turn_state.turn_delay_timer = 0.0;
        }

        // Clear placed desert tiles and return them to players
        if let Some(mut placed_tiles) = placed_tiles {
            placed_tiles.clear();
        }

        // Return desert tiles to all players
        for player in players.players.iter_mut() {
            player.has_desert_tile = true;
        }

        // Despawn visual desert tile entities
        for entity in desert_tile_entities.iter() {
            commands.entity(entity).despawn();
        }

        next_state.set(GameState::Playing);
    }
}

/// Initialize game end state with bet reveal data
pub fn setup_game_end_state(
    mut commands: Commands,
    players: Res<Players>,
    race_bets: Res<RaceBets>,
    camels: Query<(&Camel, &BoardPosition)>,
) {
    let winner = get_leading_camel(&camels);
    let loser = get_last_place_camel(&camels);

    info!("Setting up game end state. Winner: {:?}, Loser: {:?}", winner, loser);

    let mut state = GameEndState::new();
    state.winning_camel = winner;
    state.losing_camel = loser;

    // Store player scores before long-term bets
    state.scores_before_long_term = players.players.iter()
        .map(|p| (p.name.clone(), p.money, p.character_id, p.id))
        .collect();

    // Prepare winner bets for reveal
    let winner_payouts = [8, 5, 3, 2, 1];
    let mut correct_winner_count = 0;

    for (idx, bet) in race_bets.winner_bets.iter().enumerate() {
        let player = players.players.iter().find(|p| p.id == bet.player_id);
        if let Some(player) = player {
            let is_correct = Some(bet.camel) == winner;
            let payout = if is_correct {
                let p = if correct_winner_count < winner_payouts.len() {
                    winner_payouts[correct_winner_count] as i32
                } else {
                    1
                };
                correct_winner_count += 1;
                p
            } else {
                -1
            };

            state.winner_bets_to_reveal.push(PendingBetReveal {
                player_id: bet.player_id,
                player_name: player.name.clone(),
                player_character_id: player.character_id,
                camel: bet.camel,
                is_correct,
                payout,
                bet_order: idx,
            });
        }
    }

    // Prepare loser bets for reveal
    let loser_payouts = [8, 5, 3, 2, 1];
    let mut correct_loser_count = 0;

    for (idx, bet) in race_bets.loser_bets.iter().enumerate() {
        let player = players.players.iter().find(|p| p.id == bet.player_id);
        if let Some(player) = player {
            let is_correct = Some(bet.camel) == loser;
            let payout = if is_correct {
                let p = if correct_loser_count < loser_payouts.len() {
                    loser_payouts[correct_loser_count] as i32
                } else {
                    1
                };
                correct_loser_count += 1;
                p
            } else {
                -1
            };

            state.loser_bets_to_reveal.push(PendingBetReveal {
                player_id: bet.player_id,
                player_name: player.name.clone(),
                player_character_id: player.character_id,
                camel: bet.camel,
                is_correct,
                payout,
                bet_order: idx,
            });
        }
    }

    commands.insert_resource(state);
}

pub fn game_end_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut players: Option<ResMut<Players>>,
    mut game_end_state: Option<ResMut<GameEndState>>,
    mut celebration_state: Option<ResMut<CelebrationState>>,
    player_leg_bets: Option<Res<PlayerLegBetsStore>>,
    player_pyramid_tokens: Option<Res<PlayerPyramidTokens>>,
    camels: Query<(&Camel, &BoardPosition)>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref mut players) = players else { return };
    let Some(ref mut state) = game_end_state else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Update reveal timer
    state.reveal_timer += time.delta_secs();

    // Handle firework celebration during FinalResults phase
    if state.phase == GameEndPhase::FinalResults {
        if let Some(ref mut celebration) = celebration_state {
            let dt = time.delta_secs();

            // Start celebration if not active
            if !celebration.active {
                celebration.active = true;
                celebration.elapsed = 0.0;
                celebration.next_firework_time = 0.0;
                celebration.duration = 10.0; // 10 seconds of fireworks
            }

            // Spawn fireworks at intervals
            if celebration.active && celebration.elapsed < celebration.duration {
                celebration.elapsed += dt;

                if celebration.elapsed >= celebration.next_firework_time {
                    // Spawn a firework at random X position
                    let mut rng = rand::thread_rng();
                    let x_pos = rng.gen_range(-400.0..400.0);
                    let color = random_firework_color();
                    spawn_firework(&mut commands, x_pos, color);

                    // Schedule next firework (faster at the start, slower later)
                    let interval = if celebration.elapsed < 3.0 {
                        rng.gen_range(0.15..0.3)
                    } else {
                        rng.gen_range(0.3..0.5)
                    };
                    celebration.next_firework_time = celebration.elapsed + interval;
                }
            }
        }
    }

    match state.phase {
        GameEndPhase::LegComplete => {
            draw_final_leg_complete_phase(ctx, players, &player_leg_bets, &player_pyramid_tokens, &camels, state);
        }
        GameEndPhase::StandingsPreBets => {
            draw_standings_pre_bets_phase(ctx, players, state);
        }
        GameEndPhase::RevealingWinnerBets => {
            draw_winner_bets_reveal_phase(ctx, players, state);
        }
        GameEndPhase::RevealingLoserBets => {
            draw_loser_bets_reveal_phase(ctx, players, state);
        }
        GameEndPhase::FinalResults => {
            draw_final_results_phase(ctx, players, state, &mut next_state);
        }
    }
}

/// Draw the final leg complete phase (same as normal leg scoring)
fn draw_final_leg_complete_phase(
    ctx: &egui::Context,
    players: &mut ResMut<Players>,
    player_leg_bets: &Option<Res<PlayerLegBetsStore>>,
    player_pyramid_tokens: &Option<Res<PlayerPyramidTokens>>,
    camels: &Query<(&Camel, &BoardPosition)>,
    state: &mut GameEndState,
) {
    let first_place = get_leading_camel(camels);
    let second_place = get_second_place_camel(camels);

    // Calculate score changes (without applying yet if not done)
    let mut score_changes: Vec<(String, i32, Vec<(CamelColor, u8, i32)>, u8)> = Vec::new();

    if let Some(ref player_leg_bets) = player_leg_bets {
        for (player_idx, player) in players.players.iter().enumerate() {
            let mut leg_bet_total = 0i32;
            let mut bet_details: Vec<(CamelColor, u8, i32)> = Vec::new();

            if player_idx < player_leg_bets.bets.len() {
                for tile in &player_leg_bets.bets[player_idx] {
                    let change = if Some(tile.camel) == first_place {
                        tile.value as i32
                    } else if Some(tile.camel) == second_place {
                        1
                    } else {
                        -1
                    };
                    leg_bet_total += change;
                    bet_details.push((tile.camel, tile.value, change));
                }
            }

            let pyramid_tokens = if let Some(ref tokens) = player_pyramid_tokens {
                if player_idx < tokens.counts.len() {
                    tokens.counts[player_idx]
                } else {
                    0
                }
            } else {
                0
            };

            score_changes.push((player.name.clone(), leg_bet_total, bet_details, pyramid_tokens));
        }
    }

    // Apply leg scores if not done
    if !state.leg_scores_applied {
        if let Some(ref player_leg_bets) = player_leg_bets {
            for (player_idx, player) in players.players.iter_mut().enumerate() {
                if player_idx < player_leg_bets.bets.len() {
                    for tile in &player_leg_bets.bets[player_idx] {
                        if Some(tile.camel) == first_place {
                            player.money += tile.value as i32;
                        } else if Some(tile.camel) == second_place {
                            player.money += 1;
                        } else {
                            player.money = (player.money - 1).max(0);
                        }
                    }
                }
            }
        }
        state.leg_scores_applied = true;

        // Update scores_before_long_term with post-leg scores
        state.scores_before_long_term = players.players.iter()
            .map(|p| (p.name.clone(), p.money, p.character_id, p.id))
            .collect();
    }

    let mut sorted_players: Vec<_> = players.players.iter()
        .map(|p| (p.name.clone(), p.money))
        .collect();
    sorted_players.sort_by(|a, b| b.1.cmp(&a.1));

    let mut should_continue = false;

    egui::Area::new(egui::Id::new("game_end_leg_scoring"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220))
                .corner_radius(egui::CornerRadius::same(16))
                .inner_margin(egui::Margin::same(32))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 4,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
                })
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("Final Leg Complete!").size(32.0).strong());
                        ui.add_space(20.0);

                        // Show first and second place with camel silhouettes
                        if let Some(first) = first_place {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("1st Place:").size(16.0));
                                let color = camel_color_to_egui(first);
                                let border_color = egui::Color32::from_rgb(
                                    (color.r() as f32 * 0.5) as u8,
                                    (color.g() as f32 * 0.5) as u8,
                                    (color.b() as f32 * 0.5) as u8,
                                );
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 30.0), egui::Sense::hover());
                                draw_camel_silhouette(ui.painter(), rect, color, border_color);
                                ui.label(egui::RichText::new(format!("{:?}", first)).size(16.0).strong());
                            });
                        }
                        if let Some(second) = second_place {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("2nd Place:").size(16.0));
                                let color = camel_color_to_egui(second);
                                let border_color = egui::Color32::from_rgb(
                                    (color.r() as f32 * 0.5) as u8,
                                    (color.g() as f32 * 0.5) as u8,
                                    (color.b() as f32 * 0.5) as u8,
                                );
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 30.0), egui::Sense::hover());
                                draw_camel_silhouette(ui.painter(), rect, color, border_color);
                                ui.label(egui::RichText::new(format!("{:?}", second)).size(16.0).strong());
                            });
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Show score changes
                        ui.heading(egui::RichText::new("Leg Earnings").size(20.0));
                        ui.add_space(10.0);

                        for (name, leg_bet_total, details, pyramid_tokens) in &score_changes {
                            let has_bets = !details.is_empty();
                            let has_tokens = *pyramid_tokens > 0;

                            if has_bets || has_tokens {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(name).strong().size(14.0));
                                    ui.label(":");
                                    ui.add_space(8.0);

                                    for (camel, value, change) in details {
                                        let card_size = egui::vec2(28.0, 38.0);
                                        let (rect, _) = ui.allocate_exact_size(card_size, egui::Sense::hover());
                                        draw_mini_leg_bet_card(ui.painter(), rect, *camel, *value);

                                        let change_text = if *change > 0 {
                                            format!("+${}", change)
                                        } else {
                                            format!("-$1")
                                        };
                                        let change_color = if *change > 0 {
                                            egui::Color32::LIGHT_GREEN
                                        } else {
                                            egui::Color32::from_rgb(255, 100, 100)
                                        };
                                        ui.label(egui::RichText::new(&change_text).size(12.0).color(change_color));
                                        ui.add_space(4.0);
                                    }

                                    if *pyramid_tokens > 0 {
                                        if !details.is_empty() {
                                            ui.add_space(8.0);
                                        }
                                        draw_pyramid_token_icon(ui, *pyramid_tokens);
                                    }

                                    let total = *leg_bet_total + (*pyramid_tokens as i32);
                                    if total != 0 {
                                        ui.add_space(12.0);
                                        let total_text = if total > 0 {
                                            format!("= +${}", total)
                                        } else {
                                            format!("= -${}", total.abs())
                                        };
                                        let total_color = if total > 0 {
                                            egui::Color32::LIGHT_GREEN
                                        } else {
                                            egui::Color32::from_rgb(255, 100, 100)
                                        };
                                        ui.label(egui::RichText::new(&total_text).strong().size(14.0).color(total_color));
                                    }
                                });
                                ui.add_space(4.0);
                            }
                        }

                        ui.add_space(30.0);

                        if ui.button(egui::RichText::new("Continue to Final Standings").size(18.0)).clicked() {
                            should_continue = true;
                        }
                    });
                });
        });

    if should_continue {
        state.phase = GameEndPhase::StandingsPreBets;
        state.reveal_timer = 0.0;
    }
}

/// Draw the standings before long-term bets are revealed
fn draw_standings_pre_bets_phase(
    ctx: &egui::Context,
    players: &ResMut<Players>,
    state: &mut GameEndState,
) {
    let mut sorted_players: Vec<_> = players.players.iter().enumerate().collect();
    sorted_players.sort_by(|a, b| b.1.money.cmp(&a.1.money));

    let has_winner_bets = !state.winner_bets_to_reveal.is_empty();
    let has_loser_bets = !state.loser_bets_to_reveal.is_empty();
    let has_any_bets = has_winner_bets || has_loser_bets;

    let mut should_continue = false;

    egui::Area::new(egui::Id::new("game_end_standings_pre"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 230))
                .corner_radius(egui::CornerRadius::same(16))
                .inner_margin(egui::Margin::same(40))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 4,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
                })
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("Current Standings").size(36.0).strong());
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Before Long-Term Bet Results").size(16.0).color(egui::Color32::GRAY));
                        ui.add_space(30.0);

                        // Draw standings with avatars
                        for (rank, (player_idx, player)) in sorted_players.iter().enumerate() {
                            let player_color = PLAYER_COLORS[*player_idx % PLAYER_COLORS.len()];
                            let rank_text = match rank {
                                0 => "1st",
                                1 => "2nd",
                                2 => "3rd",
                                3 => "4th",
                                4 => "5th",
                                5 => "6th",
                                6 => "7th",
                                7 => "8th",
                                _ => "",
                            };

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(rank_text).size(18.0).strong().monospace());
                                ui.add_space(10.0);

                                // Avatar
                                let avatar_size = 50.0;
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(avatar_size, avatar_size), egui::Sense::hover());
                                draw_avatar(ui.painter(), rect, player.character_id, Some(player_color));

                                ui.add_space(10.0);

                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&player.name).size(16.0).strong());
                                    ui.label(egui::RichText::new(format!("${}", player.money)).size(14.0).color(egui::Color32::GOLD));
                                });
                            });
                            ui.add_space(8.0);
                        }

                        ui.add_space(30.0);

                        if has_any_bets {
                            let btn_text = if has_winner_bets {
                                "Reveal Winner Bets"
                            } else {
                                "Reveal Loser Bets"
                            };
                            if ui.button(egui::RichText::new(btn_text).size(18.0)).clicked() {
                                should_continue = true;
                            }
                        } else {
                            if ui.button(egui::RichText::new("Show Final Results").size(18.0)).clicked() {
                                state.phase = GameEndPhase::FinalResults;
                            }
                        }
                    });
                });
        });

    if should_continue {
        if has_winner_bets {
            state.phase = GameEndPhase::RevealingWinnerBets;
        } else if has_loser_bets {
            state.phase = GameEndPhase::RevealingLoserBets;
        } else {
            state.phase = GameEndPhase::FinalResults;
        }
        state.current_reveal_index = 0;
        state.reveal_timer = 0.0;
    }
}

/// Draw the winner bets reveal phase with animation
fn draw_winner_bets_reveal_phase(
    ctx: &egui::Context,
    players: &mut ResMut<Players>,
    state: &mut GameEndState,
) {
    let mut should_advance = false;
    let current_idx = state.current_reveal_index;
    let total_bets = state.winner_bets_to_reveal.len();

    // Auto-advance after animation duration
    if state.reveal_timer >= state.reveal_animation_duration && current_idx < total_bets {
        // Apply the current bet's payout
        let bet = &state.winner_bets_to_reveal[current_idx];
        if let Some(player) = players.players.iter_mut().find(|p| p.id == bet.player_id) {
            if bet.payout > 0 {
                player.money += bet.payout;
            } else {
                player.money = (player.money - 1).max(0);
            }
        }
        state.current_reveal_index += 1;
        state.reveal_timer = 0.0;
    }

    // Sort players by current money
    let mut sorted_players: Vec<_> = players.players.iter().enumerate().collect();
    sorted_players.sort_by(|a, b| b.1.money.cmp(&a.1.money));

    egui::Area::new(egui::Id::new("game_end_winner_reveal"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 230))
                .corner_radius(egui::CornerRadius::same(16))
                .inner_margin(egui::Margin::same(40))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 4,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
                })
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("Winner Bets").size(32.0).strong().color(egui::Color32::GOLD));

                        // Show winning camel
                        if let Some(winner) = state.winning_camel {
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Race Winner:").size(14.0));
                                let color = camel_color_to_egui(winner);
                                let border_color = egui::Color32::from_rgb(
                                    (color.r() as f32 * 0.5) as u8,
                                    (color.g() as f32 * 0.5) as u8,
                                    (color.b() as f32 * 0.5) as u8,
                                );
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 22.0), egui::Sense::hover());
                                draw_camel_silhouette(ui.painter(), rect, color, border_color);
                                ui.label(egui::RichText::new(format!("{:?}", winner)).size(14.0).strong());
                            });
                        }

                        ui.add_space(20.0);

                        // Current bet being revealed (or most recently revealed)
                        if current_idx < total_bets {
                            let bet = &state.winner_bets_to_reveal[current_idx];
                            draw_bet_reveal_card(ui, bet, state.reveal_timer / state.reveal_animation_duration);
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Current standings with progress bars
                        ui.label(egui::RichText::new("Current Standings").size(18.0).strong());
                        ui.add_space(10.0);

                        // Find max money for scaling progress bars
                        let max_money = sorted_players.iter().map(|(_, p)| p.money).max().unwrap_or(1).max(1);
                        let bar_max_width = 150.0;

                        for (rank, (player_idx, player)) in sorted_players.iter().enumerate() {
                            let player_color = PLAYER_COLORS[*player_idx % PLAYER_COLORS.len()];
                            ui.horizontal(|ui| {
                                let rank_text = format!("{}.", rank + 1);
                                ui.label(egui::RichText::new(&rank_text).size(14.0).monospace());

                                let avatar_size = 35.0;
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(avatar_size, avatar_size), egui::Sense::hover());
                                draw_avatar(ui.painter(), rect, player.character_id, Some(player_color));

                                ui.label(egui::RichText::new(&player.name).size(14.0));

                                ui.add_space(8.0);

                                // Money progress bar
                                let bar_width = (player.money as f32 / max_money as f32) * bar_max_width;
                                let bar_height = 16.0;
                                let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(bar_max_width, bar_height), egui::Sense::hover());

                                // Background
                                ui.painter().rect_filled(bar_rect, 4.0, egui::Color32::from_rgb(40, 40, 40));

                                // Filled portion
                                let filled_rect = egui::Rect::from_min_size(
                                    bar_rect.min,
                                    egui::vec2(bar_width, bar_height)
                                );
                                ui.painter().rect_filled(filled_rect, 4.0, egui::Color32::from_rgb(180, 150, 50));

                                // Money text on bar
                                ui.painter().text(
                                    bar_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("${}", player.money),
                                    egui::FontId::proportional(12.0),
                                    egui::Color32::WHITE,
                                );
                            });
                            ui.add_space(4.0);
                        }

                        ui.add_space(16.0);

                        // Progress indicator
                        ui.label(egui::RichText::new(format!("Bet {}/{}", (current_idx + 1).min(total_bets), total_bets)).size(12.0).color(egui::Color32::GRAY));

                        // Manual advance button (or skip to next phase)
                        if current_idx >= total_bets {
                            ui.add_space(10.0);
                            let next_text = if !state.loser_bets_to_reveal.is_empty() {
                                "Continue to Loser Bets"
                            } else {
                                "Show Final Results"
                            };
                            let button = egui::Button::new(egui::RichText::new(next_text).size(16.0))
                                .min_size(egui::vec2(200.0, 50.0));
                            if ui.add(button).clicked() {
                                should_advance = true;
                            }
                        }
                    });
                });
        });

    if should_advance {
        if !state.loser_bets_to_reveal.is_empty() {
            state.phase = GameEndPhase::RevealingLoserBets;
            state.current_reveal_index = 0;
        } else {
            state.phase = GameEndPhase::FinalResults;
        }
        state.reveal_timer = 0.0;
    }
}

/// Draw the loser bets reveal phase with animation
fn draw_loser_bets_reveal_phase(
    ctx: &egui::Context,
    players: &mut ResMut<Players>,
    state: &mut GameEndState,
) {
    let mut should_advance = false;
    let current_idx = state.current_reveal_index;
    let total_bets = state.loser_bets_to_reveal.len();

    // Auto-advance after animation duration
    if state.reveal_timer >= state.reveal_animation_duration && current_idx < total_bets {
        // Apply the current bet's payout
        let bet = &state.loser_bets_to_reveal[current_idx];
        if let Some(player) = players.players.iter_mut().find(|p| p.id == bet.player_id) {
            if bet.payout > 0 {
                player.money += bet.payout;
            } else {
                player.money = (player.money - 1).max(0);
            }
        }
        state.current_reveal_index += 1;
        state.reveal_timer = 0.0;
    }

    // Sort players by current money
    let mut sorted_players: Vec<_> = players.players.iter().enumerate().collect();
    sorted_players.sort_by(|a, b| b.1.money.cmp(&a.1.money));

    egui::Area::new(egui::Id::new("game_end_loser_reveal"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 230))
                .corner_radius(egui::CornerRadius::same(16))
                .inner_margin(egui::Margin::same(40))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 4,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
                })
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("Loser Bets").size(32.0).strong().color(egui::Color32::from_rgb(200, 100, 100)));

                        // Show losing camel
                        if let Some(loser) = state.losing_camel {
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Race Loser:").size(14.0));
                                let color = camel_color_to_egui(loser);
                                let border_color = egui::Color32::from_rgb(
                                    (color.r() as f32 * 0.5) as u8,
                                    (color.g() as f32 * 0.5) as u8,
                                    (color.b() as f32 * 0.5) as u8,
                                );
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 22.0), egui::Sense::hover());
                                draw_camel_silhouette(ui.painter(), rect, color, border_color);
                                ui.label(egui::RichText::new(format!("{:?}", loser)).size(14.0).strong());
                            });
                        }

                        ui.add_space(20.0);

                        // Current bet being revealed
                        if current_idx < total_bets {
                            let bet = &state.loser_bets_to_reveal[current_idx];
                            draw_bet_reveal_card(ui, bet, state.reveal_timer / state.reveal_animation_duration);
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Current standings with progress bars
                        ui.label(egui::RichText::new("Current Standings").size(18.0).strong());
                        ui.add_space(10.0);

                        // Find max money for scaling progress bars
                        let max_money = sorted_players.iter().map(|(_, p)| p.money).max().unwrap_or(1).max(1);
                        let bar_max_width = 150.0;

                        for (rank, (player_idx, player)) in sorted_players.iter().enumerate() {
                            let player_color = PLAYER_COLORS[*player_idx % PLAYER_COLORS.len()];
                            ui.horizontal(|ui| {
                                let rank_text = format!("{}.", rank + 1);
                                ui.label(egui::RichText::new(&rank_text).size(14.0).monospace());

                                let avatar_size = 35.0;
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(avatar_size, avatar_size), egui::Sense::hover());
                                draw_avatar(ui.painter(), rect, player.character_id, Some(player_color));

                                ui.label(egui::RichText::new(&player.name).size(14.0));

                                ui.add_space(8.0);

                                // Money progress bar
                                let bar_width = (player.money as f32 / max_money as f32) * bar_max_width;
                                let bar_height = 16.0;
                                let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(bar_max_width, bar_height), egui::Sense::hover());

                                // Background
                                ui.painter().rect_filled(bar_rect, 4.0, egui::Color32::from_rgb(40, 40, 40));

                                // Filled portion
                                let filled_rect = egui::Rect::from_min_size(
                                    bar_rect.min,
                                    egui::vec2(bar_width, bar_height)
                                );
                                ui.painter().rect_filled(filled_rect, 4.0, egui::Color32::from_rgb(180, 150, 50));

                                // Money text on bar
                                ui.painter().text(
                                    bar_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("${}", player.money),
                                    egui::FontId::proportional(12.0),
                                    egui::Color32::WHITE,
                                );
                            });
                            ui.add_space(4.0);
                        }

                        ui.add_space(16.0);

                        // Progress indicator
                        ui.label(egui::RichText::new(format!("Bet {}/{}", (current_idx + 1).min(total_bets), total_bets)).size(12.0).color(egui::Color32::GRAY));

                        // Manual advance button
                        if current_idx >= total_bets {
                            ui.add_space(10.0);
                            let button = egui::Button::new(egui::RichText::new("Show Final Results").size(16.0))
                                .min_size(egui::vec2(200.0, 50.0));
                            if ui.add(button).clicked() {
                                should_advance = true;
                            }
                        }
                    });
                });
        });

    if should_advance {
        state.phase = GameEndPhase::FinalResults;
        state.reveal_timer = 0.0;
    }
}

/// Draw the final results with winner announcement
fn draw_final_results_phase(
    ctx: &egui::Context,
    players: &ResMut<Players>,
    _state: &GameEndState,
    next_state: &mut ResMut<NextState<GameState>>,
) {
    // Sort players by money
    let mut sorted_players: Vec<_> = players.players.iter().enumerate().collect();
    sorted_players.sort_by(|a, b| b.1.money.cmp(&a.1.money));

    let winner = sorted_players.first().map(|(_, p)| p);

    egui::Area::new(egui::Id::new("game_end_final"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 240))
                .corner_radius(egui::CornerRadius::same(16))
                .inner_margin(egui::Margin::same(50))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 4,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
                })
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("GAME OVER!").size(48.0).strong().color(egui::Color32::GOLD));
                        ui.add_space(30.0);

                        // Winner announcement
                        if let Some(winner) = winner {
                            let winner_idx = players.players.iter().position(|p| p.id == winner.id).unwrap_or(0);
                            let winner_color = PLAYER_COLORS[winner_idx % PLAYER_COLORS.len()];

                            ui.horizontal(|ui| {
                                // Winner avatar (large) - with happy expression!
                                let avatar_size = 80.0;
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(avatar_size, avatar_size), egui::Sense::hover());
                                draw_avatar_with_expression(ui.painter(), rect, winner.character_id, Some(winner_color), true);

                                ui.add_space(20.0);

                                ui.vertical(|ui| {
                                    ui.heading(egui::RichText::new(format!("{} WINS!", winner.name)).size(36.0).strong().color(egui::Color32::WHITE));
                                    ui.label(egui::RichText::new(format!("with {} Egyptian Pounds!", winner.money)).size(20.0).color(egui::Color32::GOLD));
                                });
                            });
                        }

                        ui.add_space(40.0);
                        ui.separator();
                        ui.add_space(20.0);

                        ui.heading(egui::RichText::new("Final Standings").size(24.0));
                        ui.add_space(15.0);

                        // Final standings with avatars
                        for (rank, (player_idx, player)) in sorted_players.iter().enumerate() {
                            let player_color = PLAYER_COLORS[*player_idx % PLAYER_COLORS.len()];
                            let is_winner = rank == 0;

                            let rank_text = match rank {
                                0 => "1st",
                                1 => "2nd",
                                2 => "3rd",
                                3 => "4th",
                                4 => "5th",
                                5 => "6th",
                                6 => "7th",
                                7 => "8th",
                                _ => "",
                            };

                            ui.horizontal(|ui| {
                                // Fixed width for rank text to align columns
                                ui.label(egui::RichText::new(rank_text).size(16.0).monospace());

                                let avatar_size = 45.0;
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(avatar_size, avatar_size), egui::Sense::hover());
                                // Winner gets a happy expression!
                                draw_avatar_with_expression(ui.painter(), rect, player.character_id, Some(player_color), is_winner);

                                ui.add_space(10.0);

                                let name_text = if is_winner {
                                    egui::RichText::new(&player.name).size(16.0).strong().color(egui::Color32::GOLD)
                                } else {
                                    egui::RichText::new(&player.name).size(16.0)
                                };
                                ui.label(name_text);

                                let ai_tag = if player.is_ai { " (AI)" } else { "" };
                                ui.label(egui::RichText::new(ai_tag).size(12.0).color(egui::Color32::GRAY));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let money_color = if is_winner { egui::Color32::GOLD } else { egui::Color32::WHITE };
                                    ui.label(egui::RichText::new(format!("${}", player.money)).size(16.0).strong().color(money_color));
                                });
                            });
                            ui.add_space(6.0);
                        }

                        ui.add_space(40.0);

                        ui.horizontal(|ui| {
                            if ui.button(egui::RichText::new("Play Again").size(20.0)).clicked() {
                                next_state.set(GameState::MainMenu);
                            }

                            ui.add_space(20.0);

                            if ui.button(egui::RichText::new("Quit").size(16.0)).clicked() {
                                std::process::exit(0);
                            }
                        });
                    });
                });
        });
}

/// Draw a bet reveal card with animation - card only shows camel color and avatar
/// The payout text is displayed separately beside the card
fn draw_bet_reveal_card(ui: &mut egui::Ui, bet: &PendingBetReveal, progress: f32) {
    let camel_color = camel_color_to_egui(bet.camel);

    // Animate scale and opacity
    let scale = 0.5 + progress.min(1.0) * 0.5;
    let alpha = (progress * 2.0).min(1.0);

    ui.horizontal(|ui| {
        // Card part
        let card_width = 120.0 * scale;
        let card_height = 80.0 * scale;

        let (rect, _) = ui.allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::hover());

        // Card background with camel color
        let bg_alpha = (alpha * 255.0) as u8;
        let bg_color = egui::Color32::from_rgba_unmultiplied(
            camel_color.r(),
            camel_color.g(),
            camel_color.b(),
            bg_alpha,
        );
        ui.painter().rect_filled(rect, 8.0 * scale, bg_color);

        // Border
        let border_color = if bet.is_correct {
            egui::Color32::from_rgba_unmultiplied(100, 255, 100, bg_alpha)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 100, 100, bg_alpha)
        };
        ui.painter().rect_stroke(rect, 8.0 * scale, egui::Stroke::new(3.0 * scale, border_color), egui::epaint::StrokeKind::Outside);

        // Avatar centered on card
        let avatar_size = 50.0 * scale;
        let avatar_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2((card_width - avatar_size) / 2.0, (card_height - avatar_size) / 2.0),
            egui::vec2(avatar_size, avatar_size),
        );
        draw_avatar(ui.painter(), avatar_rect, bet.player_character_id, None);

        // Correct/Wrong indicator on card
        let text_alpha = (alpha * 255.0) as u8;
        let indicator = if bet.is_correct { "✓" } else { "✗" };
        let indicator_color = if bet.is_correct {
            egui::Color32::from_rgba_unmultiplied(100, 255, 100, text_alpha)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 100, 100, text_alpha)
        };
        ui.painter().text(
            egui::pos2(rect.max.x - 12.0 * scale, rect.min.y + 12.0 * scale),
            egui::Align2::CENTER_CENTER,
            indicator,
            egui::FontId::proportional(16.0 * scale),
            indicator_color,
        );

        ui.add_space(15.0);

        // Payout text beside the card: "Player Name: +$8"
        let result_text = if bet.is_correct {
            format!("{}: +${}", bet.player_name, bet.payout)
        } else {
            format!("{}: -$1", bet.player_name)
        };
        let result_color = if bet.is_correct {
            egui::Color32::from_rgba_unmultiplied(100, 255, 100, text_alpha)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 100, 100, text_alpha)
        };

        ui.vertical(|ui| {
            ui.add_space(card_height / 2.0 - 12.0);
            ui.label(egui::RichText::new(&result_text)
                .size(20.0 * scale)
                .strong()
                .color(result_color));
        });
    });
}

/// Helper to draw pyramid token icons - one icon per token collected
fn draw_pyramid_token_icon(ui: &mut egui::Ui, count: u8) {
    use crate::ui::hud::draw_pyramid_token_icon as draw_single_token;

    let token_size = 20.0;
    let token_spacing = 16.0; // Slight overlap for stacked look

    // Allocate space for all tokens
    let total_width = token_size + (count.saturating_sub(1) as f32 * token_spacing);
    let (tokens_rect, _) = ui.allocate_exact_size(
        egui::vec2(total_width, token_size),
        egui::Sense::hover()
    );

    // Draw each token
    for t in 0..count {
        let token_center = egui::pos2(
            tokens_rect.left() + token_size / 2.0 + (t as f32 * token_spacing),
            tokens_rect.center().y
        );
        draw_single_token(ui.painter(), token_center, token_size);
    }

    // Show total value
    ui.label(egui::RichText::new(format!("+${}", count))
        .size(12.0).color(egui::Color32::GOLD));
}
