use bevy::prelude::*;
use crate::components::*;
use crate::game::state::GameState;
use crate::systems::turn::{TurnState, PlayerLegBetsStore, PlayerPyramidTokens};
use crate::systems::movement::{get_leading_camel, get_second_place_camel, get_last_place_camel};

/// Message to trigger leg scoring display
#[derive(Message)]
pub struct LegScoringComplete {
    pub scores: Vec<(String, i32)>, // (player_name, score_change)
}

/// System to calculate and apply leg scores
pub fn calculate_leg_scores(
    mut players: ResMut<Players>,
    player_leg_bets: Res<PlayerLegBetsStore>,
    player_pyramid_tokens: Res<PlayerPyramidTokens>,
    camels: Query<(&Camel, &BoardPosition)>,
    mut scoring_complete: MessageWriter<LegScoringComplete>,
) {
    let first_place = get_leading_camel(&camels);
    let second_place = get_second_place_camel(&camels);

    info!("Leg scoring: 1st place: {:?}, 2nd place: {:?}", first_place, second_place);

    let mut score_changes: Vec<(String, i32)> = Vec::new();

    for (player_idx, player) in players.players.iter_mut().enumerate() {
        let mut score_change = 0i32;

        // Score leg betting tiles
        if player_idx < player_leg_bets.bets.len() {
            for tile in &player_leg_bets.bets[player_idx] {
                if Some(tile.camel) == first_place {
                    // Bet on first place - earn tile value
                    score_change += tile.value as i32;
                } else if Some(tile.camel) == second_place {
                    // Bet on second place - earn 1
                    score_change += 1;
                } else {
                    // Wrong bet - lose 1
                    score_change -= 1;
                }
            }
        }

        // Score pyramid tokens (1 coin each, already given during rolling)
        // The +1 was already given when rolling, so nothing to do here

        player.money += score_change;
        // Ensure money doesn't go below 0
        if player.money < 0 {
            player.money = 0;
        }

        score_changes.push((player.name.clone(), score_change));
    }

    scoring_complete.write(LegScoringComplete { scores: score_changes });
}

/// System to reset for a new leg
pub fn reset_for_new_leg(
    mut pyramid: ResMut<Pyramid>,
    mut leg_tiles: ResMut<LegBettingTiles>,
    mut player_leg_bets: ResMut<PlayerLegBetsStore>,
    mut player_pyramid_tokens: ResMut<PlayerPyramidTokens>,
    mut turn_state: ResMut<TurnState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Reset pyramid (return all dice)
    pyramid.reset();

    // Reset leg betting tiles
    leg_tiles.reset();

    // Clear player leg bets
    player_leg_bets.clear_all();

    // Clear pyramid token counts
    player_pyramid_tokens.clear_all();

    // Increment leg number
    turn_state.leg_number += 1;
    turn_state.action_taken = false;
    turn_state.awaiting_action = true;

    info!("Starting leg {}", turn_state.leg_number);

    // Return to playing state
    next_state.set(GameState::Playing);
}

/// System to calculate final game scores
pub fn calculate_final_scores(
    mut players: ResMut<Players>,
    race_bets: Res<RaceBets>,
    camels: Query<(&Camel, &BoardPosition)>,
) {
    let winner = get_leading_camel(&camels);
    let loser = get_last_place_camel(&camels);

    info!("Game over! Winner: {:?}, Loser: {:?}", winner, loser);

    // Score winner bets
    let winner_payouts = [8, 5, 3, 2, 1];
    let mut winner_payout_idx = 0;

    for bet in &race_bets.winner_bets {
        if Some(bet.camel) == winner {
            // Correct bet
            let payout = if winner_payout_idx < winner_payouts.len() {
                winner_payouts[winner_payout_idx]
            } else {
                1
            };
            winner_payout_idx += 1;

            if let Some(player) = players.players.iter_mut().find(|p| p.id == bet.player_id) {
                player.money += payout;
                info!("{} earned {} for correct winner bet on {:?}", player.name, payout, bet.camel);
            }
        } else {
            // Wrong bet
            if let Some(player) = players.players.iter_mut().find(|p| p.id == bet.player_id) {
                player.money = (player.money - 1).max(0);
                info!("{} lost 1 for wrong winner bet on {:?}", player.name, bet.camel);
            }
        }
    }

    // Score loser bets
    let loser_payouts = [8, 5, 3, 2, 1];
    let mut loser_payout_idx = 0;

    for bet in &race_bets.loser_bets {
        if Some(bet.camel) == loser {
            // Correct bet
            let payout = if loser_payout_idx < loser_payouts.len() {
                loser_payouts[loser_payout_idx]
            } else {
                1
            };
            loser_payout_idx += 1;

            if let Some(player) = players.players.iter_mut().find(|p| p.id == bet.player_id) {
                player.money += payout;
                info!("{} earned {} for correct loser bet on {:?}", player.name, payout, bet.camel);
            }
        } else {
            // Wrong bet
            if let Some(player) = players.players.iter_mut().find(|p| p.id == bet.player_id) {
                player.money = (player.money - 1).max(0);
                info!("{} lost 1 for wrong loser bet on {:?}", player.name, bet.camel);
            }
        }
    }

    // Determine winner
    let game_winner = players.players.iter().max_by_key(|p| p.money);
    if let Some(winner) = game_winner {
        info!("GAME WINNER: {} with {} coins!", winner.name, winner.money);
    }
}
