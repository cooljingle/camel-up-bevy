//! State synchronization between game instances via Firebase

use bevy::prelude::*;
use crate::components::{
    BoardPosition, Camel, CamelColor, CrazyCamel, CrazyCamelColor, Players,
    LegBettingTiles, RaceBets, PlacedSpectatorTiles, Pyramid, GameBoard,
};
use crate::systems::turn::{TurnState, PlayerLegBetsStore, PlayerPyramidTokens};
use super::state::{NetworkState, ReceivedGameState, PendingNetworkActions};
use super::messages::*;
use super::js_bindings;

/// System to poll Firebase for updates
pub fn poll_firebase_updates(
    network_state: Res<NetworkState>,
    mut received_state: ResMut<ReceivedGameState>,
    mut pending_actions: ResMut<PendingNetworkActions>,
) {
    if !network_state.is_online() {
        return;
    }

    // Poll for game state updates (clients receive these)
    if network_state.is_client() {
        if let Some(state_json) = js_bindings::poll_game_state() {
            info!("Client received game state update ({} bytes)", state_json.len());
            received_state.state_json = Some(state_json);
            received_state.needs_processing = true;
        }
    }

    // Poll for action updates (host receives these)
    if network_state.is_host() {
        if let Some(actions_json) = js_bindings::poll_actions() {
            if let Ok(actions) = serde_json::from_str::<Vec<serde_json::Value>>(&actions_json) {
                for action_value in actions {
                    if let Ok(action) = serde_json::from_value::<NetworkActionMessage>(action_value) {
                        pending_actions.actions.push(action);
                    }
                }
            }
        }
    }
}

/// System to apply received game state (clients only)
pub fn process_received_game_state(
    network_state: Res<NetworkState>,
    mut received_state: ResMut<ReceivedGameState>,
    mut players: Option<ResMut<Players>>,
    mut turn_state: Option<ResMut<TurnState>>,
    mut pyramid: Option<ResMut<Pyramid>>,
    mut leg_betting_tiles: Option<ResMut<LegBettingTiles>>,
    mut race_bets: Option<ResMut<RaceBets>>,
    mut placed_tiles: Option<ResMut<PlacedSpectatorTiles>>,
    mut player_leg_bets: Option<ResMut<PlayerLegBetsStore>>,
    mut player_pyramid_tokens: Option<ResMut<PlayerPyramidTokens>>,
    mut camels: Query<(&Camel, &mut BoardPosition, &mut Transform)>,
    mut crazy_camels: Query<(&CrazyCamel, &mut BoardPosition, &mut Transform), Without<Camel>>,
) {
    if !network_state.is_client() || !received_state.needs_processing {
        return;
    }

    let Some(ref state_json) = received_state.state_json else {
        return;
    };

    let state = match serde_json::from_str::<SerializableGameState>(state_json) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to parse received game state: {}", e);
            // Log a preview of the JSON to see structure
            let preview_len = state_json.len().min(500);
            warn!("JSON preview: {}", &state_json[..preview_len]);
            return;
        }
    };

    // Only process if version is newer
    if state.version <= received_state.version {
        received_state.needs_processing = false;
        return;
    }

    info!("Applying game state version {} with {} camels, {} crazy camels",
        state.version, state.camels.len(), state.crazy_camels.len());
    received_state.version = state.version;
    received_state.needs_processing = false;

    // Apply turn state
    if let Some(ref mut ts) = turn_state {
        ts.current_player = state.turn_state.current_player;
        ts.action_taken = state.turn_state.action_taken;
        ts.leg_number = state.turn_state.leg_number;
        ts.awaiting_action = state.turn_state.awaiting_action;
        ts.leg_has_started = state.turn_state.leg_has_started;
    }

    // Apply player data
    if let Some(ref mut p) = players {
        p.current_player_index = state.turn_state.current_player;
        for (i, sp) in state.players.iter().enumerate() {
            if let Some(player) = p.players.get_mut(i) {
                player.money = sp.money;
                player.has_spectator_tile = sp.has_spectator_tile;
                player.available_race_cards = sp.available_race_cards
                    .iter()
                    .filter_map(|c| parse_camel_color(c))
                    .collect();
            }
        }
    }

    // Apply camel positions (both BoardPosition and visual Transform)
    // Get board for position calculations
    let board = crate::components::GameBoard::new();

    for (camel, mut pos, mut transform) in camels.iter_mut() {
        if let Some(sp) = state.camels.iter().find(|c| c.color == format!("{:?}", camel.color)) {
            let old_pos = (pos.space_index, pos.stack_position);
            pos.space_index = sp.space_index;
            pos.stack_position = sp.stack_position;

            // Update visual position too (snap to board position)
            let base_pos = board.get_position(sp.space_index);
            let stack_offset = sp.stack_position as f32 * 25.0;
            let old_transform = transform.translation;
            transform.translation.x = base_pos.x;
            transform.translation.y = base_pos.y + stack_offset;
            transform.translation.z = 10.0 + sp.stack_position as f32;

            if old_pos != (sp.space_index, sp.stack_position) {
                info!("Moved {:?} camel from {:?} to ({}, {}), transform {:?} -> {:?}",
                    camel.color, old_pos, sp.space_index, sp.stack_position,
                    old_transform, transform.translation);
            }
        }
    }

    // Apply crazy camel positions
    for (camel, mut pos, mut transform) in crazy_camels.iter_mut() {
        if let Some(sp) = state.crazy_camels.iter().find(|c| c.color == format!("{:?}", camel.color)) {
            pos.space_index = sp.space_index;
            pos.stack_position = sp.stack_position;

            // Update visual position too
            let base_pos = board.get_position(sp.space_index);
            let stack_offset = sp.stack_position as f32 * 25.0;
            transform.translation.x = base_pos.x;
            transform.translation.y = base_pos.y + stack_offset;
            transform.translation.z = 10.0 + sp.stack_position as f32;
        }
    }

    // Apply pyramid state
    if let Some(ref mut pyr) = pyramid {
        pyr.rolled_dice.clear();
        for die in &state.pyramid.rolled_dice {
            if die.is_crazy {
                if let Some(color) = parse_crazy_camel_color(&die.color) {
                    pyr.rolled_dice.push(crate::components::dice::PyramidDie::Crazy {
                        rolled: Some((color, die.value)),
                    });
                }
            } else {
                if let Some(color) = parse_camel_color(&die.color) {
                    let mut regular_die = crate::components::dice::RegularDie::new(color);
                    regular_die.value = Some(die.value);
                    pyr.rolled_dice.push(crate::components::dice::PyramidDie::Regular(regular_die));
                }
            }
        }
    }

    // Apply player leg bets
    if let Some(ref mut plb) = player_leg_bets {
        for (i, bets) in state.player_leg_bets.iter().enumerate() {
            if i < plb.bets.len() {
                plb.bets[i].clear();
                for bet in bets {
                    if let Some(color) = parse_camel_color(&bet.camel_color) {
                        plb.bets[i].push(crate::components::LegBetTile {
                            camel: color,
                            value: bet.value,
                        });
                    }
                }
            }
        }
    }

    // Apply player pyramid tokens
    if let Some(ref mut ppt) = player_pyramid_tokens {
        for (i, &count) in state.player_pyramid_tokens.iter().enumerate() {
            if i < ppt.counts.len() {
                ppt.counts[i] = count;
            }
        }
    }
}

/// System to broadcast game state to Firebase (host only)
pub fn broadcast_game_state_system(
    network_state: Res<NetworkState>,
    players: Res<Players>,
    turn_state: Res<TurnState>,
    pyramid: Res<Pyramid>,
    leg_betting_tiles: Res<LegBettingTiles>,
    race_bets: Res<RaceBets>,
    placed_tiles: Res<PlacedSpectatorTiles>,
    player_leg_bets: Res<PlayerLegBetsStore>,
    player_pyramid_tokens: Res<PlayerPyramidTokens>,
    camels: Query<(&Camel, &BoardPosition)>,
    crazy_camels: Query<(&CrazyCamel, &BoardPosition), Without<Camel>>,
    mut last_version: Local<u32>,
) {
    if !network_state.is_host() {
        return;
    }

    let Some(ref room_code) = network_state.room_code else {
        return;
    };

    // Create serializable state
    let version = *last_version + 1;

    let state = SerializableGameState {
        version,
        turn_state: SerializableTurnState {
            current_player: turn_state.current_player,
            action_taken: turn_state.action_taken,
            leg_number: turn_state.leg_number,
            awaiting_action: turn_state.awaiting_action,
            leg_has_started: turn_state.leg_has_started,
        },
        players: players.players.iter().enumerate().map(|(i, p)| {
            SerializablePlayer {
                id: p.id,
                network_id: network_state.local_player_id.clone().unwrap_or_default(), // TODO: Map properly
                name: p.name.clone(),
                money: p.money,
                has_spectator_tile: p.has_spectator_tile,
                available_race_cards: p.available_race_cards.iter().map(|c| format!("{:?}", c)).collect(),
                is_ai: p.is_ai,
                character_id: p.character_id as u8,
                color_index: p.color_index,
            }
        }).collect(),
        camels: camels.iter().map(|(c, p)| {
            SerializableCamelPosition {
                color: format!("{:?}", c.color),
                space_index: p.space_index,
                stack_position: p.stack_position,
            }
        }).collect(),
        crazy_camels: crazy_camels.iter().map(|(c, p)| {
            SerializableCamelPosition {
                color: format!("{:?}", c.color),
                space_index: p.space_index,
                stack_position: p.stack_position,
            }
        }).collect(),
        pyramid: SerializablePyramid {
            rolled_dice: pyramid.rolled_dice.iter().filter_map(|r| {
                match r {
                    crate::components::dice::PyramidDie::Regular(die) => {
                        die.value.map(|value| SerializableDieResult {
                            color: format!("{:?}", die.color),
                            value,
                            is_crazy: false,
                        })
                    }
                    crate::components::dice::PyramidDie::Crazy { rolled } => {
                        rolled.map(|(color, value)| SerializableDieResult {
                            color: format!("{:?}", color),
                            value,
                            is_crazy: true,
                        })
                    }
                }
            }).collect(),
        },
        leg_betting_tiles: SerializableLegBettingTiles {
            tiles: CamelColor::all().iter().enumerate().map(|(i, color)| {
                let available: Vec<u8> = leg_betting_tiles.stacks.get(i)
                    .map(|stack| stack.iter().map(|t| t.value).collect())
                    .unwrap_or_default();
                (format!("{:?}", color), available)
            }).collect(),
        },
        winner_bets: race_bets.winner_bets.iter().map(|b| {
            SerializableRaceBet {
                camel_color: format!("{:?}", b.camel),
                player_id: b.player_id,
            }
        }).collect(),
        loser_bets: race_bets.loser_bets.iter().map(|b| {
            SerializableRaceBet {
                camel_color: format!("{:?}", b.camel),
                player_id: b.player_id,
            }
        }).collect(),
        placed_spectator_tiles: placed_tiles.tiles.iter().map(|(&space, &(owner, is_oasis))| {
            SerializableSpectatorTile {
                space_index: space,
                owner_id: owner,
                is_oasis,
            }
        }).collect(),
        player_leg_bets: player_leg_bets.bets.iter().map(|bets| {
            bets.iter().map(|b| SerializableLegBet {
                camel_color: format!("{:?}", b.camel),
                value: b.value,
            }).collect()
        }).collect(),
        player_pyramid_tokens: player_pyramid_tokens.counts.clone(),
    };

    // Serialize and send
    if let Ok(json) = serde_json::to_string(&state) {
        js_bindings::async_ops::write_state_async(room_code.clone(), json);
        *last_version = version;
    }
}

/// Parse a camel color from a string
fn parse_camel_color(s: &str) -> Option<CamelColor> {
    match s {
        "Blue" => Some(CamelColor::Blue),
        "Green" => Some(CamelColor::Green),
        "Red" => Some(CamelColor::Red),
        "Yellow" => Some(CamelColor::Yellow),
        "Purple" => Some(CamelColor::Purple),
        _ => None,
    }
}

/// Parse a crazy camel color from a string
fn parse_crazy_camel_color(s: &str) -> Option<CrazyCamelColor> {
    match s {
        "Black" => Some(CrazyCamelColor::Black),
        "White" => Some(CrazyCamelColor::White),
        _ => None,
    }
}
