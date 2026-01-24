use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rand::Rng;
use crate::components::{Players, CamelColor, Camel, BoardPosition, RaceBets};
use crate::game::state::GameState;
use crate::systems::movement::{get_leading_camel, get_second_place_camel, get_last_place_camel};
use crate::systems::turn::{PlayerLegBetsStore, PlayerPyramidTokens};
use crate::systems::animation::{spawn_firework, random_firework_color};
use crate::ui::characters::{draw_avatar, draw_avatar_with_expression, draw_avatar_crown};
use crate::ui::hud::{draw_camel_silhouette, draw_crown_overlay, draw_dunce_cap_overlay, draw_mini_leg_bet_card};
use crate::ui::theme::{camel_color_to_egui, desert_button, DesertButtonStyle, PLAYER_COLORS};

/// Easing function for smooth panel animations
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

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
    pub grey_hold_duration: f32,     // Time to show grey card before flipping
    pub coin_update_delay: f32,  // Delay after flip before updating coins
    pub current_payout_applied: bool,  // Track if current card's payout has been applied
    pub winning_camel: Option<CamelColor>,
    pub losing_camel: Option<CamelColor>,
    // Track scores before long-term bets are applied
    pub scores_before_long_term: Vec<(String, i32, crate::ui::characters::CharacterId, u8)>, // (name, money, character_id, player_id)
    // Animation progress for mobile panels (0.0 = hidden, 1.0 = fully visible)
    pub panel_animation_progress: f32,
    // Animated money values for smooth progress bar updates (indexed by player_id)
    pub animated_player_money: Vec<f32>,
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
            reveal_animation_duration: 0.7, // Total time: 500ms grey hold + 200ms flip
            grey_hold_duration: 0.5,        // Show grey card for 500ms before flipping
            coin_update_delay: 0.4,         // Wait 400ms after flip before updating coins
            current_payout_applied: false,
            winning_camel: None,
            losing_camel: None,
            scores_before_long_term: Vec::new(),
            panel_animation_progress: 0.0,
            animated_player_money: Vec::new(),
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

    // Initialize animated money values from current player money
    // Use player_id as index, so we need to find the max player_id
    let max_player_id = players.players.iter().map(|p| p.id).max().unwrap_or(0) as usize;
    state.animated_player_money = vec![0.0; max_player_id + 1];
    for player in &players.players {
        state.animated_player_money[player.id as usize] = player.money as f32;
    }

    // Prepare winner bets for reveal
    let winner_payouts = [8, 5, 3, 2, 1];
    let mut correct_winner_count = 0;

    for (_idx, bet) in race_bets.winner_bets.iter().enumerate() {
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
            });
        }
    }

    // Prepare loser bets for reveal
    let loser_payouts = [8, 5, 3, 2, 1];
    let mut correct_loser_count = 0;

    for (_idx, bet) in race_bets.loser_bets.iter().enumerate() {
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
    ui_state: Res<crate::ui::hud::UiState>,
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

            // Spawn fireworks at intervals - continues indefinitely until new game
            if celebration.active {
                celebration.elapsed += dt;

                if celebration.elapsed >= celebration.next_firework_time {
                    // Spawn a firework at random X position
                    let mut rng = rand::thread_rng();
                    let x_pos = rng.gen_range(-400.0..400.0);
                    let color = random_firework_color();
                    spawn_firework(&mut commands, x_pos, color);

                    // Schedule next firework (faster at the start, then steady pace)
                    let interval = if celebration.elapsed < 3.0 {
                        rng.gen_range(0.15..0.3)
                    } else {
                        rng.gen_range(0.4..0.7)
                    };
                    celebration.next_firework_time = celebration.elapsed + interval;
                }
            }
        }
    }

    let is_mobile = !ui_state.use_side_panels;

    match state.phase {
        GameEndPhase::LegComplete => {
            draw_final_leg_complete_phase(ctx, players, &player_leg_bets, &player_pyramid_tokens, &camels, state, is_mobile);
        }
        GameEndPhase::StandingsPreBets => {
            draw_standings_pre_bets_phase(ctx, players, state);
        }
        GameEndPhase::RevealingWinnerBets => {
            draw_winner_bets_reveal_phase(ctx, players, state, is_mobile, time.delta_secs());
        }
        GameEndPhase::RevealingLoserBets => {
            draw_loser_bets_reveal_phase(ctx, players, state, is_mobile, time.delta_secs());
        }
        GameEndPhase::FinalResults => {
            draw_final_results_phase(ctx, players, state, &mut next_state, is_mobile, time.delta_secs());
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
    is_mobile: bool,
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
                        // Only show heading on desktop to save vertical space on mobile
                        if !is_mobile {
                            ui.heading(egui::RichText::new("Leg Earnings").size(20.0));
                            ui.add_space(10.0);
                        }

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

                        if desert_button(ui, "Final Standings", &DesertButtonStyle::medium()).clicked() {
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
                        ui.horizontal(|ui| {
                            ui.add_space(5.0);
                            ui.heading(egui::RichText::new("Current Standings").size(36.0).strong());
                        });
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Before Long-Term Bet Results").size(16.0).color(egui::Color32::GRAY));
                        ui.add_space(30.0);

                        // Draw standings with avatars
                        for (rank, (_player_idx, player)) in sorted_players.iter().enumerate() {
                            let player_color = PLAYER_COLORS[player.color_index % PLAYER_COLORS.len()];
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
                                "To Winner Bets"
                            } else {
                                "To Loser Bets"
                            };
                            if desert_button(ui, btn_text, &DesertButtonStyle::medium()).clicked() {
                                should_continue = true;
                            }
                        } else {
                            if desert_button(ui, "Final Results", &DesertButtonStyle::medium()).clicked() {
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
    is_mobile: bool,
    delta: f32,
) {
    let mut should_advance = false;
    let mut should_next_card = false;
    let current_idx = state.current_reveal_index;
    let total_bets = state.winner_bets_to_reveal.len();

    // Check animation state
    let payout_time = state.reveal_timer >= state.reveal_animation_duration + state.coin_update_delay;

    // Apply payout once after animation + delay (but don't auto-advance)
    if payout_time && current_idx < total_bets && !state.current_payout_applied {
        let bet = &state.winner_bets_to_reveal[current_idx];
        if let Some(player) = players.players.iter_mut().find(|p| p.id == bet.player_id) {
            if bet.payout > 0 {
                player.money += bet.payout;
            } else {
                player.money = (player.money - 1).max(0);
            }
        }
        state.current_payout_applied = true;
    }

    // Animate money values towards actual player money (lerp each frame)
    let lerp_speed = 5.0; // Speed of animation
    for player in players.players.iter() {
        if (player.id as usize) < state.animated_player_money.len() {
            let target = player.money as f32;
            let current = state.animated_player_money[player.id as usize];
            let diff = target - current;
            if diff.abs() > 0.5 {
                state.animated_player_money[player.id as usize] = current + diff * (lerp_speed * delta).min(1.0);
            } else {
                state.animated_player_money[player.id as usize] = target;
            }
        }
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
                                draw_crown_overlay(ui.painter(), rect);  // Winner wears a crown
                                ui.label(egui::RichText::new(format!("{:?}", winner)).size(14.0).strong());
                            });
                        }

                        ui.add_space(20.0);

                        // Current bet being revealed (or most recently revealed)
                        if current_idx < total_bets {
                            let bet = &state.winner_bets_to_reveal[current_idx];
                            let grey_hold_ratio = state.grey_hold_duration / state.reveal_animation_duration;
                            draw_bet_reveal_card(ui, bet, state.reveal_timer / state.reveal_animation_duration, grey_hold_ratio);
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Current standings with progress bars
                        // Only show heading on desktop to save vertical space on mobile
                        if !is_mobile {
                            ui.horizontal(|ui| {
                                ui.add_space(5.0);
                                ui.label(egui::RichText::new("Current Standings").size(18.0).strong());
                            });
                            ui.add_space(10.0);
                        }

                        // Find max money for scaling progress bars - use max($50, max_player_score)
                        let actual_max = sorted_players.iter().map(|(_, p)| p.money).max().unwrap_or(1);
                        let max_money = actual_max.max(50).max(1);
                        let bar_max_width = 150.0;

                        for (rank, (_player_idx, player)) in sorted_players.iter().enumerate() {
                            let player_color = PLAYER_COLORS[player.color_index % PLAYER_COLORS.len()];
                            ui.horizontal(|ui| {
                                let rank_text = format!("{}.", rank + 1);
                                ui.label(egui::RichText::new(&rank_text).size(14.0).monospace());

                                let avatar_size = 35.0;
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(avatar_size, avatar_size), egui::Sense::hover());
                                draw_avatar(ui.painter(), rect, player.character_id, Some(player_color));

                                ui.label(egui::RichText::new(&player.name).size(14.0));

                                ui.add_space(8.0);

                                // Money progress bar - use animated value for smooth transitions
                                let animated_money = if (player.id as usize) < state.animated_player_money.len() {
                                    state.animated_player_money[player.id as usize]
                                } else {
                                    player.money as f32
                                };
                                let bar_width = (animated_money / max_money as f32) * bar_max_width;
                                let bar_height = 16.0;
                                let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(bar_max_width, bar_height), egui::Sense::hover());

                                // Filled portion (no background)
                                let filled_rect = egui::Rect::from_min_size(
                                    bar_rect.min,
                                    egui::vec2(bar_width.max(0.0), bar_height)
                                );
                                ui.painter().rect_filled(filled_rect, 4.0, egui::Color32::from_rgb(180, 150, 50));

                                // Money text on filled bar - show animated value rounded
                                ui.painter().text(
                                    filled_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("${}", animated_money.round() as i32),
                                    egui::FontId::proportional(12.0),
                                    egui::Color32::WHITE,
                                );
                            });
                            ui.add_space(4.0);
                        }

                        ui.add_space(16.0);

                        // Progress indicator
                        ui.label(egui::RichText::new(format!("Bet {}/{}", (current_idx + 1).min(total_bets), total_bets)).size(12.0).color(egui::Color32::GRAY));

                        // Next button - always rendered to prevent layout shift, but invisible when not ready
                        ui.add_space(10.0);
                        let btn_visible = current_idx < total_bets && state.current_payout_applied;
                        let btn_text = if current_idx + 1 < total_bets {
                            "Next"
                        } else if !state.loser_bets_to_reveal.is_empty() {
                            "Continue to Loser Bets"
                        } else {
                            "Finish Game"
                        };

                        // Use scope to modify opacity without affecting other UI
                        ui.scope(|ui| {
                            if !btn_visible {
                                // Make button invisible but still occupy space
                                ui.set_invisible();
                            }
                            if desert_button(ui, btn_text, &DesertButtonStyle::default()).clicked() && btn_visible {
                                if current_idx + 1 < total_bets {
                                    should_next_card = true;
                                } else {
                                    should_advance = true;
                                }
                            }
                        });
                    });
                });
        });

    if should_next_card {
        state.current_reveal_index += 1;
        state.reveal_timer = 0.0;
        state.current_payout_applied = false;
    }

    if should_advance {
        if !state.loser_bets_to_reveal.is_empty() {
            state.phase = GameEndPhase::RevealingLoserBets;
            state.current_reveal_index = 0;
        } else {
            state.phase = GameEndPhase::FinalResults;
        }
        state.reveal_timer = 0.0;
        state.current_payout_applied = false;
    }
}

/// Draw the loser bets reveal phase with animation
fn draw_loser_bets_reveal_phase(
    ctx: &egui::Context,
    players: &mut ResMut<Players>,
    state: &mut GameEndState,
    is_mobile: bool,
    delta: f32,
) {
    let mut should_advance = false;
    let mut should_next_card = false;
    let current_idx = state.current_reveal_index;
    let total_bets = state.loser_bets_to_reveal.len();

    // Check animation state
    let payout_time = state.reveal_timer >= state.reveal_animation_duration + state.coin_update_delay;

    // Apply payout once after animation + delay (but don't auto-advance)
    if payout_time && current_idx < total_bets && !state.current_payout_applied {
        let bet = &state.loser_bets_to_reveal[current_idx];
        if let Some(player) = players.players.iter_mut().find(|p| p.id == bet.player_id) {
            if bet.payout > 0 {
                player.money += bet.payout;
            } else {
                player.money = (player.money - 1).max(0);
            }
        }
        state.current_payout_applied = true;
    }

    // Animate money values towards actual player money (lerp each frame)
    let lerp_speed = 5.0; // Speed of animation
    for player in players.players.iter() {
        if (player.id as usize) < state.animated_player_money.len() {
            let target = player.money as f32;
            let current = state.animated_player_money[player.id as usize];
            let diff = target - current;
            if diff.abs() > 0.5 {
                state.animated_player_money[player.id as usize] = current + diff * (lerp_speed * delta).min(1.0);
            } else {
                state.animated_player_money[player.id as usize] = target;
            }
        }
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
                                draw_dunce_cap_overlay(ui.painter(), rect);  // Loser wears a dunce cap
                                ui.label(egui::RichText::new(format!("{:?}", loser)).size(14.0).strong());
                            });
                        }

                        ui.add_space(20.0);

                        // Current bet being revealed
                        if current_idx < total_bets {
                            let bet = &state.loser_bets_to_reveal[current_idx];
                            let grey_hold_ratio = state.grey_hold_duration / state.reveal_animation_duration;
                            draw_bet_reveal_card(ui, bet, state.reveal_timer / state.reveal_animation_duration, grey_hold_ratio);
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Current standings with progress bars
                        // Only show heading on desktop to save vertical space on mobile
                        if !is_mobile {
                            ui.horizontal(|ui| {
                                ui.add_space(5.0);
                                ui.label(egui::RichText::new("Current Standings").size(18.0).strong());
                            });
                            ui.add_space(10.0);
                        }

                        // Find max money for scaling progress bars - use max($50, max_player_score)
                        let actual_max = sorted_players.iter().map(|(_, p)| p.money).max().unwrap_or(1);
                        let max_money = actual_max.max(50).max(1);
                        let bar_max_width = 150.0;

                        for (rank, (_player_idx, player)) in sorted_players.iter().enumerate() {
                            let player_color = PLAYER_COLORS[player.color_index % PLAYER_COLORS.len()];
                            ui.horizontal(|ui| {
                                let rank_text = format!("{}.", rank + 1);
                                ui.label(egui::RichText::new(&rank_text).size(14.0).monospace());

                                let avatar_size = 35.0;
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(avatar_size, avatar_size), egui::Sense::hover());
                                draw_avatar(ui.painter(), rect, player.character_id, Some(player_color));

                                ui.label(egui::RichText::new(&player.name).size(14.0));

                                ui.add_space(8.0);

                                // Money progress bar - use animated value for smooth transitions
                                let animated_money = if (player.id as usize) < state.animated_player_money.len() {
                                    state.animated_player_money[player.id as usize]
                                } else {
                                    player.money as f32
                                };
                                let bar_width = (animated_money / max_money as f32) * bar_max_width;
                                let bar_height = 16.0;
                                let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(bar_max_width, bar_height), egui::Sense::hover());

                                // Filled portion (no background)
                                let filled_rect = egui::Rect::from_min_size(
                                    bar_rect.min,
                                    egui::vec2(bar_width.max(0.0), bar_height)
                                );
                                ui.painter().rect_filled(filled_rect, 4.0, egui::Color32::from_rgb(180, 150, 50));

                                // Money text on filled bar - show animated value rounded
                                ui.painter().text(
                                    filled_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("${}", animated_money.round() as i32),
                                    egui::FontId::proportional(12.0),
                                    egui::Color32::WHITE,
                                );
                            });
                            ui.add_space(4.0);
                        }

                        ui.add_space(16.0);

                        // Progress indicator
                        ui.label(egui::RichText::new(format!("Bet {}/{}", (current_idx + 1).min(total_bets), total_bets)).size(12.0).color(egui::Color32::GRAY));

                        // Next button - always rendered to prevent layout shift, but invisible when not ready
                        ui.add_space(10.0);
                        let btn_visible = current_idx < total_bets && state.current_payout_applied;
                        let btn_text = if current_idx + 1 < total_bets {
                            "Next"
                        } else {
                            "Show Final Results"
                        };

                        // Use scope to modify opacity without affecting other UI
                        ui.scope(|ui| {
                            if !btn_visible {
                                // Make button invisible but still occupy space
                                ui.set_invisible();
                            }
                            if desert_button(ui, btn_text, &DesertButtonStyle::default()).clicked() && btn_visible {
                                if current_idx + 1 < total_bets {
                                    should_next_card = true;
                                } else {
                                    should_advance = true;
                                }
                            }
                        });
                    });
                });
        });

    if should_next_card {
        state.current_reveal_index += 1;
        state.reveal_timer = 0.0;
        state.current_payout_applied = false;
    }

    if should_advance {
        state.phase = GameEndPhase::FinalResults;
        state.reveal_timer = 0.0;
        state.current_payout_applied = false;
    }
}

/// Draw the final results using sliding panels on mobile
/// Top panel: Final standings list
/// Bottom panel: Winner announcement and action buttons
fn draw_final_results_mobile_panels(
    ctx: &egui::Context,
    _players: &ResMut<Players>,
    sorted_players: &[(usize, &crate::components::player::PlayerData)],
    state: &mut GameEndState,
    next_state: &mut ResMut<NextState<GameState>>,
    time_delta: f32,
) {
    // Animate panel progress (0 to 1 over 0.3 seconds)
    state.panel_animation_progress = (state.panel_animation_progress + time_delta / 0.3).min(1.0);
    let progress = ease_out_cubic(state.panel_animation_progress);

    let winner = sorted_players.first().map(|(idx, p)| (*idx, *p));

    // Top panel: slides down from top - Final Standings
    egui::TopBottomPanel::top("final_standings_top")
        .frame(egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, (230.0 * progress) as u8))
            .inner_margin(egui::Margin::symmetric(12, 8)))
        .show_animated(ctx, progress > 0.01, |ui| {
            // Title
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("Final Standings")
                    .size(22.0)
                    .strong()
                    .color(egui::Color32::WHITE));
            });
            ui.add_space(6.0);

            // Player rankings (compact horizontal layout for mobile)
            for (rank, (_player_idx, player)) in sorted_players.iter().enumerate() {
                let player_color = PLAYER_COLORS[player.color_index % PLAYER_COLORS.len()];
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
                    // Rank
                    ui.label(egui::RichText::new(rank_text).size(13.0).monospace());

                    // Avatar
                    let avatar_size = 24.0;
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(avatar_size, avatar_size),
                        egui::Sense::hover()
                    );
                    draw_avatar_with_expression(ui.painter(), rect, player.character_id, Some(player_color), is_winner);
                    if is_winner {
                        draw_avatar_crown(ui.painter(), rect);
                    }

                    ui.add_space(4.0);

                    // Name
                    let name_text = if is_winner {
                        egui::RichText::new(&player.name).size(13.0).strong().color(egui::Color32::GOLD)
                    } else {
                        egui::RichText::new(&player.name).size(13.0)
                    };
                    ui.label(name_text);

                    // Money (right-aligned)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let money_color = if is_winner { egui::Color32::GOLD } else { egui::Color32::WHITE };
                        ui.label(egui::RichText::new(format!("${}", player.money))
                            .size(13.0)
                            .strong()
                            .color(money_color));
                    });
                });
                ui.add_space(2.0);
            }
        });

    // Bottom panel: slides up from bottom - Winner + Buttons
    egui::TopBottomPanel::bottom("final_standings_bottom")
        .frame(egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, (230.0 * progress) as u8))
            .inner_margin(egui::Margin::symmetric(12, 10)))
        .show_animated(ctx, progress > 0.01, |ui| {
            ui.vertical_centered(|ui| {
                // Winner announcement
                if let Some((_winner_idx, winner)) = winner {
                    let winner_color = PLAYER_COLORS[winner.color_index % PLAYER_COLORS.len()];

                    ui.horizontal(|ui| {
                        // Winner avatar
                        let avatar_size = 40.0;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(avatar_size, avatar_size),
                            egui::Sense::hover()
                        );
                        draw_avatar_with_expression(ui.painter(), rect, winner.character_id, Some(winner_color), true);
                        draw_avatar_crown(ui.painter(), rect);

                        ui.add_space(8.0);

                        ui.vertical(|ui| {
                            ui.heading(egui::RichText::new(format!("{} Wins!", winner.name))
                                .size(20.0)
                                .strong()
                                .color(egui::Color32::GOLD));
                            ui.label(egui::RichText::new(format!("with ${}", winner.money))
                                .size(14.0)
                                .color(egui::Color32::WHITE));
                        });
                    });
                }

                ui.add_space(12.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if desert_button(ui, "Play Again", &DesertButtonStyle::default()).clicked() {
                        next_state.set(GameState::MainMenu);
                    }

                    ui.add_space(12.0);

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if desert_button(ui, "Quit", &DesertButtonStyle::default()).clicked() {
                            std::process::exit(0);
                        }
                    }
                });
            });
        });
}

/// Draw the final results with winner announcement
fn draw_final_results_phase(
    ctx: &egui::Context,
    players: &ResMut<Players>,
    state: &mut GameEndState,
    next_state: &mut ResMut<NextState<GameState>>,
    is_mobile: bool,
    time_delta: f32,
) {
    // Sort players by money
    let mut sorted_players: Vec<_> = players.players.iter().enumerate().collect();
    sorted_players.sort_by(|a, b| b.1.money.cmp(&a.1.money));

    // Use sliding panels on mobile, modal on desktop
    if is_mobile {
        draw_final_results_mobile_panels(ctx, players, &sorted_players, state, next_state, time_delta);
        return;
    }

    // Desktop: existing modal implementation
    let winner = sorted_players.first().map(|(_, p)| p);

    // Responsive sizes (desktop only now)
    let (title_size, winner_name_size, subtitle_size, body_size) = (32.0, 26.0, 18.0, 16.0);
    let margin = 40;
    let winner_avatar_size = 55.0;
    let standings_avatar_size = 36.0;

    egui::Area::new(egui::Id::new("game_end_final"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 230))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::same(margin as i8))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 4],
                    blur: 16,
                    spread: 2,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120),
                })
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        // Title - Final Standings at top
                        ui.heading(egui::RichText::new("Final Standings").size(title_size).strong().color(egui::Color32::WHITE));
                        ui.add_space(if is_mobile { 12.0 } else { 20.0 });

                        // Final standings with avatars
                        for (rank, (_player_idx, player)) in sorted_players.iter().enumerate() {
                            let player_color = PLAYER_COLORS[player.color_index % PLAYER_COLORS.len()];
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
                                ui.label(egui::RichText::new(rank_text).size(body_size).monospace());

                                let (rect, _) = ui.allocate_exact_size(egui::vec2(standings_avatar_size, standings_avatar_size), egui::Sense::hover());
                                // Winner gets a happy expression!
                                draw_avatar_with_expression(ui.painter(), rect, player.character_id, Some(player_color), is_winner);
                                if is_winner {
                                    draw_avatar_crown(ui.painter(), rect);
                                }

                                ui.add_space(if is_mobile { 5.0 } else { 10.0 });

                                let name_text = if is_winner {
                                    egui::RichText::new(&player.name).size(body_size).strong().color(egui::Color32::GOLD)
                                } else {
                                    egui::RichText::new(&player.name).size(body_size)
                                };
                                ui.label(name_text);

                                if !is_mobile {
                                    let ai_tag = if player.is_ai { " (AI)" } else { "" };
                                    ui.label(egui::RichText::new(ai_tag).size(12.0).color(egui::Color32::GRAY));
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let money_color = if is_winner { egui::Color32::GOLD } else { egui::Color32::WHITE };
                                    ui.label(egui::RichText::new(format!("${}", player.money)).size(body_size).strong().color(money_color));
                                });
                            });
                            ui.add_space(if is_mobile { 3.0 } else { 5.0 });
                        }

                        ui.add_space(if is_mobile { 15.0 } else { 25.0 });
                        ui.separator();
                        ui.add_space(if is_mobile { 15.0 } else { 20.0 });

                        // Winner announcement at bottom
                        if let Some(winner) = winner {
                            let winner_color = PLAYER_COLORS[winner.color_index % PLAYER_COLORS.len()];

                            ui.horizontal(|ui| {
                                // Winner avatar - with happy expression!
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(winner_avatar_size, winner_avatar_size), egui::Sense::hover());
                                draw_avatar_with_expression(ui.painter(), rect, winner.character_id, Some(winner_color), true);
                                draw_avatar_crown(ui.painter(), rect);

                                ui.add_space(if is_mobile { 8.0 } else { 15.0 });

                                ui.vertical(|ui| {
                                    ui.heading(egui::RichText::new(format!("{} Wins!", winner.name)).size(winner_name_size).strong().color(egui::Color32::GOLD));
                                    ui.label(egui::RichText::new(format!("with ${}", winner.money)).size(subtitle_size).color(egui::Color32::WHITE));
                                });
                            });
                        }

                        ui.add_space(if is_mobile { 20.0 } else { 25.0 });

                        // Styled buttons
                        ui.horizontal(|ui| {
                            let style = if is_mobile {
                                DesertButtonStyle::default()
                            } else {
                                DesertButtonStyle::medium()
                            };

                            if desert_button(ui, "Play Again", &style).clicked() {
                                next_state.set(GameState::MainMenu);
                            }

                            ui.add_space(if is_mobile { 15.0 } else { 20.0 });

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                if desert_button(ui, "Quit", &DesertButtonStyle::default()).clicked() {
                                    std::process::exit(0);
                                }
                            }
                        });
                    });
                });
        });
}

/// Draw a bet reveal card with flip animation
/// Phase 1 (0 to grey_hold_ratio): Grey neutral card at full size
/// Phase 2 (grey_hold_ratio to 1.0): Flip animation (grey shrinks, color grows)
fn draw_bet_reveal_card(ui: &mut egui::Ui, bet: &PendingBetReveal, progress: f32, grey_hold_ratio: f32) {
    let card_width = 70.0;   // Match race bet card picker dimensions
    let card_height = 90.0;

    // Card flip animation using horizontal scale
    // 0.0 to grey_hold_ratio: grey card at full scale (hold phase)
    // grey_hold_ratio to 1.0: flip animation
    let (is_front, scale_x) = if progress < grey_hold_ratio {
        // Hold phase - show grey card at full size
        (false, 1.0)
    } else {
        // Flip phase - normalize progress for the flip portion
        let flip_progress = (progress - grey_hold_ratio) / (1.0 - grey_hold_ratio);
        if flip_progress < 0.5 {
            (false, 1.0 - flip_progress * 2.0)  // Back of card shrinking
        } else {
            (true, (flip_progress - 0.5) * 2.0)  // Front of card growing
        }
    };

    let scaled_width = card_width * scale_x.max(0.02);  // Avoid zero width

    ui.horizontal(|ui| {
        // Allocate full card space for consistent layout
        let (allocated_rect, _) = ui.allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::hover());

        // Center the scaled card within allocated space
        let display_rect = egui::Rect::from_center_size(
            allocated_rect.center(),
            egui::vec2(scaled_width, card_height)
        );

        if is_front {
            // Revealed card - camel color with player avatar
            draw_revealed_bet_card(ui.painter(), display_rect, bet, scale_x);
        } else {
            // Grey neutral card back
            draw_card_back(ui.painter(), display_rect, scale_x);
        }

        ui.add_space(15.0);

        // Payout text beside the card (only show after flip completes)
        if is_front && scale_x > 0.5 {
            let text_alpha = ((scale_x - 0.5) * 2.0 * 255.0) as u8;  // Fade in as card expands
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
                    .size(20.0)
                    .strong()
                    .color(result_color));
            });
        }
    });
}

/// Draw the grey neutral card back (for flip animation)
fn draw_card_back(painter: &egui::Painter, rect: egui::Rect, scale_x: f32) {
    // Grey card background
    let bg_color = egui::Color32::from_rgb(100, 100, 110);
    let border_color = egui::Color32::from_rgb(70, 70, 80);

    painter.rect_filled(rect, 8.0, bg_color);
    painter.rect_stroke(rect, 8.0, egui::Stroke::new(2.0, border_color), egui::epaint::StrokeKind::Outside);

    // Question mark in center (scale with card width)
    if scale_x > 0.3 {
        let text_alpha = ((scale_x - 0.3) / 0.7 * 255.0) as u8;
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "?",
            egui::FontId::proportional(30.0),
            egui::Color32::from_rgba_unmultiplied(150, 150, 160, text_alpha),
        );
    }
}

/// Draw the revealed bet card with camel color and player avatar
fn draw_revealed_bet_card(painter: &egui::Painter, rect: egui::Rect, bet: &PendingBetReveal, scale_x: f32) {
    let camel_color = camel_color_to_egui(bet.camel);

    // Card background with camel color
    painter.rect_filled(rect, 8.0, camel_color);

    // Simple dark border (no green/red - payout text shows result)
    let border_color = egui::Color32::from_rgb(40, 40, 50);
    painter.rect_stroke(rect, 8.0, egui::Stroke::new(2.0, border_color), egui::epaint::StrokeKind::Outside);

    // Avatar centered on card (scale width with flip)
    let avatar_size = 50.0;
    let avatar_width = avatar_size * scale_x;
    let avatar_rect = egui::Rect::from_center_size(
        rect.center(),
        egui::vec2(avatar_width, avatar_size),
    );
    if scale_x > 0.3 {
        draw_avatar(painter, avatar_rect, bet.player_character_id, None);
    }

    // Correct/Wrong indicator in top-right corner (with proper inset)
    if scale_x > 0.5 {
        let indicator = if bet.is_correct { "✓" } else { "✗" };
        let indicator_color = if bet.is_correct {
            egui::Color32::from_rgb(100, 255, 100)
        } else {
            egui::Color32::from_rgb(255, 100, 100)
        };
        // Fixed inset from edge to avoid glitch
        painter.text(
            egui::pos2(rect.max.x - 12.0, rect.min.y + 12.0),
            egui::Align2::CENTER_CENTER,
            indicator,
            egui::FontId::proportional(14.0),
            indicator_color,
        );
    }
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
