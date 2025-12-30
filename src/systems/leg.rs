use bevy::prelude::*;
use crate::components::*;
use crate::systems::movement::{get_leading_camel, get_last_place_camel};

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

    // Winner determination happens in the UI scoring phase after bet animations complete
}
