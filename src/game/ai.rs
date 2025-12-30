// AI player logic for Camel Up
// Phase 7 implementation

use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::systems::movement::get_leading_camel;
use crate::systems::turn::{
    TurnState, RollPyramidAction, TakeLegBetAction, PlaceRaceBetAction, PlaceSpectatorTileAction,
};
use crate::ui::hud::UiState;

/// AI difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiDifficulty {
    /// Picks random valid actions
    Random,
    /// Simple heuristics - bet on leaders, roll when unsure
    #[default]
    Basic,
    /// Probability estimation and strategic play
    Smart,
}

/// Configuration for AI players
#[derive(Resource)]
pub struct AiConfig {
    pub difficulty: AiDifficulty,
    /// Delay in seconds before AI takes action (so player can see what's happening)
    pub think_delay: f32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            difficulty: AiDifficulty::Basic,
            think_delay: 1.0, // 1 second delay
        }
    }
}

/// Tracks when the AI started "thinking" for the current turn
#[derive(Resource, Default)]
pub struct AiThinkTimer {
    pub started: bool,
    pub elapsed: f32,
}

/// Available actions the AI can choose from
#[derive(Debug, Clone)]
enum AiAction {
    RollPyramid,
    TakeLegBet(CamelColor),
    PlaceRaceBet { color: CamelColor, is_winner: bool },
    PlaceSpectatorTile { space: u8, is_oasis: bool },
}

/// Main AI decision system - runs when it's an AI player's turn
pub fn ai_decision_system(
    players: Res<Players>,
    turn_state: Res<TurnState>,
    ai_config: Res<AiConfig>,
    mut ai_timer: ResMut<AiThinkTimer>,
    time: Res<Time>,
    camels: Query<(&Camel, &BoardPosition)>,
    crazy_camels: Query<(&CrazyCamel, &BoardPosition)>,
    leg_tiles: Res<LegBettingTiles>,
    pyramid: Res<Pyramid>,
    placed_tiles: Res<PlacedSpectatorTiles>,
    ui_state: Res<UiState>,
    mut roll_action: MessageWriter<RollPyramidAction>,
    mut leg_bet_action: MessageWriter<TakeLegBetAction>,
    mut race_bet_action: MessageWriter<PlaceRaceBetAction>,
    mut spectator_action: MessageWriter<PlaceSpectatorTileAction>,
) {
    // Don't act during initial roll animations
    if !ui_state.initial_rolls_complete {
        return;
    }

    // Don't act while leg scoring modal is showing
    if ui_state.show_leg_scoring {
        ai_timer.started = false;
        ai_timer.elapsed = 0.0;
        return;
    }

    // Only act if it's an AI player's turn and no action taken yet
    if turn_state.action_taken {
        // Reset timer when action is taken
        ai_timer.started = false;
        ai_timer.elapsed = 0.0;
        return;
    }

    let current = players.current_player();
    if !current.is_ai {
        // Reset timer for human players
        ai_timer.started = false;
        ai_timer.elapsed = 0.0;
        return;
    }

    // Start or update the think timer
    if !ai_timer.started {
        ai_timer.started = true;
        ai_timer.elapsed = 0.0;
        return; // Wait until next frame to start counting
    }

    ai_timer.elapsed += time.delta_secs();

    // Wait for the think delay before taking action
    if ai_timer.elapsed < ai_config.think_delay {
        return;
    }

    // Collect available actions
    let available_actions = collect_available_actions(
        current,
        &camels,
        &crazy_camels,
        &leg_tiles,
        &pyramid,
        &placed_tiles,
    );

    if available_actions.is_empty() {
        // Fallback: always can roll pyramid (unless all dice rolled, but then leg ends)
        roll_action.write(RollPyramidAction);
        return;
    }

    // Choose action based on difficulty
    let chosen_action = match ai_config.difficulty {
        AiDifficulty::Random => choose_random_action(&available_actions),
        AiDifficulty::Basic => choose_basic_action(&available_actions, &camels, &leg_tiles, &pyramid),
        AiDifficulty::Smart => choose_smart_action(&available_actions, &camels, &leg_tiles, &pyramid, current),
    };

    // Execute the chosen action
    execute_action(
        chosen_action,
        &mut roll_action,
        &mut leg_bet_action,
        &mut race_bet_action,
        &mut spectator_action,
    );
}

/// Collect all valid actions the AI can take
fn collect_available_actions(
    player: &PlayerData,
    camels: &Query<(&Camel, &BoardPosition)>,
    crazy_camels: &Query<(&CrazyCamel, &BoardPosition)>,
    leg_tiles: &LegBettingTiles,
    pyramid: &Pyramid,
    placed_tiles: &PlacedSpectatorTiles,
) -> Vec<AiAction> {
    let mut actions = Vec::new();

    // Can always roll pyramid if dice remain
    if !pyramid.all_dice_rolled() {
        actions.push(AiAction::RollPyramid);
    }

    // Check available leg betting tiles
    for color in CamelColor::all() {
        if leg_tiles.top_tile(color).is_some() {
            actions.push(AiAction::TakeLegBet(color));
        }
    }

    // Check race betting cards
    for &color in &player.available_race_cards {
        actions.push(AiAction::PlaceRaceBet { color, is_winner: true });
        actions.push(AiAction::PlaceRaceBet { color, is_winner: false });
    }

    // Check spectator tile placement
    if player.has_spectator_tile {
        let valid_spaces = get_valid_spectator_spaces(camels, crazy_camels, placed_tiles);
        for space in valid_spaces {
            actions.push(AiAction::PlaceSpectatorTile { space, is_oasis: true });
            actions.push(AiAction::PlaceSpectatorTile { space, is_oasis: false });
        }
    }

    actions
}

/// Get valid spaces where a spectator tile can be placed
fn get_valid_spectator_spaces(
    camels: &Query<(&Camel, &BoardPosition)>,
    crazy_camels: &Query<(&CrazyCamel, &BoardPosition)>,
    placed_tiles: &PlacedSpectatorTiles,
) -> Vec<u8> {
    let mut valid = Vec::new();

    // Check spaces 1-15 (can't place on 0)
    for space in 1..TRACK_LENGTH {
        // Can't place if there's already a tile
        if placed_tiles.is_space_occupied(space) {
            continue;
        }

        // Can't place if there's a camel on the space (including crazy camels)
        let has_camel = camels.iter().any(|(_, pos)| pos.space_index == space);
        let has_crazy_camel = crazy_camels.iter().any(|(_, pos)| pos.space_index == space);
        if has_camel || has_crazy_camel {
            continue;
        }

        valid.push(space);
    }

    valid
}

/// Random AI: Pick any action randomly
fn choose_random_action(actions: &[AiAction]) -> AiAction {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..actions.len());
    actions[index].clone()
}

/// Basic AI: Simple heuristics
fn choose_basic_action(
    actions: &[AiAction],
    camels: &Query<(&Camel, &BoardPosition)>,
    leg_tiles: &LegBettingTiles,
    pyramid: &Pyramid,
) -> AiAction {
    let mut rng = rand::thread_rng();

    // Get leading camel
    let leader = get_leading_camel(camels);

    // Priority 1: If leader has 5-value tile, take it
    if let Some(leader_color) = leader {
        if let Some(tile) = leg_tiles.top_tile(leader_color) {
            if tile.value == 5 {
                for action in actions {
                    if let AiAction::TakeLegBet(color) = action {
                        if *color == leader_color {
                            return action.clone();
                        }
                    }
                }
            }
        }
    }

    // Priority 2: 50% chance to take leg bet on leader if available
    if let Some(leader_color) = leader {
        if rng.gen_bool(0.5) {
            for action in actions {
                if let AiAction::TakeLegBet(color) = action {
                    if *color == leader_color {
                        return action.clone();
                    }
                }
            }
        }
    }

    // Priority 3: If most dice rolled (leg ending soon), consider race bets
    let dice_remaining = pyramid.remaining_dice_count();
    if dice_remaining <= 2 {
        // Consider placing race bets
        if let Some(leader_color) = leader {
            for action in actions {
                if let AiAction::PlaceRaceBet { color, is_winner: true } = action {
                    if *color == leader_color && rng.gen_bool(0.3) {
                        return action.clone();
                    }
                }
            }
        }
    }

    // Default: Roll pyramid die (guaranteed +1 coin)
    for action in actions {
        if matches!(action, AiAction::RollPyramid) {
            return action.clone();
        }
    }

    // Fallback: random action
    choose_random_action(actions)
}

/// Smart AI: Probability-based decision making
fn choose_smart_action(
    actions: &[AiAction],
    camels: &Query<(&Camel, &BoardPosition)>,
    leg_tiles: &LegBettingTiles,
    pyramid: &Pyramid,
    player: &PlayerData,
) -> AiAction {
    let mut rng = rand::thread_rng();

    // Get camel rankings
    let rankings = get_camel_rankings(camels);
    let leader = rankings.first().map(|(c, _, _)| *c);
    let last = rankings.last().map(|(c, _, _)| *c);

    // Get unrolled dice colors (camels that can still move)
    let unrolled_colors: Vec<CamelColor> = pyramid.dice.iter()
        .filter_map(|d| match d {
            crate::components::dice::PyramidDie::Regular(die) => Some(die.color),
            crate::components::dice::PyramidDie::Crazy { .. } => None,
        })
        .collect();

    // Calculate expected values for leg bets
    let mut best_leg_bet: Option<(AiAction, f32)> = None;

    for action in actions {
        if let AiAction::TakeLegBet(color) = action {
            if let Some(tile) = leg_tiles.top_tile(*color) {
                let ev = calculate_leg_bet_ev(*color, tile.value, &rankings, &unrolled_colors);
                if ev > 0.5 {
                    // Only consider positive expected value bets
                    if best_leg_bet.is_none() || ev > best_leg_bet.as_ref().unwrap().1 {
                        best_leg_bet = Some((action.clone(), ev));
                    }
                }
            }
        }
    }

    // If we have a good leg bet (EV > 1.5), take it
    if let Some((action, ev)) = &best_leg_bet {
        if *ev > 1.5 {
            return action.clone();
        }
    }

    // Consider race bets in later stages
    let dice_rolled = 5 - pyramid.remaining_dice_count();
    let game_progress = dice_rolled as f32 / 5.0;

    // Only consider race bets if we've seen some dice and have strong leader
    if game_progress > 0.4 && !player.available_race_cards.is_empty() {
        if let Some(leader_color) = leader {
            // Check if leader is far ahead
            if let Some((_, leader_space, _)) = rankings.first() {
                if let Some((_, second_space, _)) = rankings.get(1) {
                    let lead = leader_space - second_space;
                    if lead >= 2 && rng.gen_bool(0.4) {
                        // Strong lead - consider winner bet
                        for action in actions {
                            if let AiAction::PlaceRaceBet { color, is_winner: true } = action {
                                if *color == leader_color {
                                    return action.clone();
                                }
                            }
                        }
                    }
                }
            }
        }

        // Consider loser bets on last place camel
        if let Some(last_color) = last {
            if let Some((_, last_space, _)) = rankings.last() {
                if let Some((_, second_last_space, _)) = rankings.get(rankings.len().saturating_sub(2)) {
                    let behind = second_last_space - last_space;
                    if behind >= 2 && rng.gen_bool(0.3) {
                        for action in actions {
                            if let AiAction::PlaceRaceBet { color, is_winner: false } = action {
                                if *color == last_color {
                                    return action.clone();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Take good leg bet if available
    if let Some((action, _)) = best_leg_bet {
        return action;
    }

    // Consider strategic spectator tile placement
    if player.has_spectator_tile {
        if let Some(_leader_color) = leader {
            if let Some((_, leader_space, _)) = rankings.first() {
                // Place oasis 2-3 spaces ahead of leader
                let target_space = leader_space + 2;
                if target_space < TRACK_LENGTH {
                    for action in actions {
                        if let AiAction::PlaceSpectatorTile { space, is_oasis: true } = action {
                            if *space == target_space || *space == target_space + 1 {
                                if rng.gen_bool(0.3) {
                                    return action.clone();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Default: Roll pyramid
    for action in actions {
        if matches!(action, AiAction::RollPyramid) {
            return action.clone();
        }
    }

    choose_random_action(actions)
}

/// Get camel rankings sorted by position (first = leader)
fn get_camel_rankings(camels: &Query<(&Camel, &BoardPosition)>) -> Vec<(CamelColor, u8, u8)> {
    let mut rankings: Vec<(CamelColor, u8, u8)> = camels
        .iter()
        .map(|(c, p)| (c.color, p.space_index, p.stack_position))
        .collect();

    // Sort by space descending, then stack position descending
    rankings.sort_by(|a, b| {
        b.1.cmp(&a.1).then(b.2.cmp(&a.2))
    });

    rankings
}

/// Calculate expected value of a leg bet
fn calculate_leg_bet_ev(
    color: CamelColor,
    tile_value: u8,
    rankings: &[(CamelColor, u8, u8)],
    unrolled_colors: &[CamelColor],
) -> f32 {
    // Find current position
    let position = rankings.iter().position(|(c, _, _)| *c == color);
    let Some(pos) = position else { return -1.0 };

    // Check if this camel can still move
    let can_move = unrolled_colors.contains(&color);

    // Simple probability estimation
    let p_first: f32;
    let p_second: f32;

    match pos {
        0 => {
            // Currently first
            p_first = if can_move { 0.7 } else { 0.5 };
            p_second = if can_move { 0.2 } else { 0.3 };
        }
        1 => {
            // Currently second
            p_first = if can_move { 0.35 } else { 0.25 };
            p_second = if can_move { 0.35 } else { 0.35 };
        }
        2 => {
            // Currently third
            p_first = if can_move { 0.15 } else { 0.1 };
            p_second = if can_move { 0.25 } else { 0.2 };
        }
        _ => {
            // Fourth or fifth
            p_first = if can_move { 0.08 } else { 0.05 };
            p_second = if can_move { 0.15 } else { 0.1 };
        }
    }

    let p_other = 1.0 - p_first - p_second;

    // EV = P(1st) * tile_value + P(2nd) * 1 - P(other) * 1
    p_first * tile_value as f32 + p_second * 1.0 - p_other * 1.0
}

/// Execute the chosen AI action
fn execute_action(
    action: AiAction,
    roll_action: &mut MessageWriter<RollPyramidAction>,
    leg_bet_action: &mut MessageWriter<TakeLegBetAction>,
    race_bet_action: &mut MessageWriter<PlaceRaceBetAction>,
    spectator_action: &mut MessageWriter<PlaceSpectatorTileAction>,
) {
    match action {
        AiAction::RollPyramid => {
            info!("AI chose to roll pyramid");
            roll_action.write(RollPyramidAction);
        }
        AiAction::TakeLegBet(color) => {
            info!("AI chose to take {:?} leg bet", color);
            leg_bet_action.write(TakeLegBetAction { color });
        }
        AiAction::PlaceRaceBet { color, is_winner } => {
            let bet_type = if is_winner { "winner" } else { "loser" };
            info!("AI chose to bet on {:?} as {}", color, bet_type);
            race_bet_action.write(PlaceRaceBetAction {
                color,
                is_winner_bet: is_winner,
            });
        }
        AiAction::PlaceSpectatorTile { space, is_oasis } => {
            let tile_type = if is_oasis { "oasis" } else { "mirage" };
            info!("AI chose to place {} on space {}", tile_type, space + 1);
            spectator_action.write(PlaceSpectatorTileAction {
                space_index: space,
                is_oasis,
            });
        }
    }
}
