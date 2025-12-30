use bevy::prelude::*;
use crate::components::*;
use crate::components::dice::DieRollResult;
use crate::game::state::GameState;
use crate::systems::movement::MovementCompleteEvent;
use crate::systems::animation::{DiceRollAnimation, DiceSprite, PendingCamelMove, PendingCrazyCamelMove, MovementAnimation, spawn_crown};

/// The current game turn state
#[derive(Resource)]
pub struct TurnState {
    pub current_player: usize,
    pub action_taken: bool,
    pub leg_number: u32,
    pub awaiting_action: bool,
    pub leg_has_started: bool, // Set to true after first action in a leg
    pub turn_delay_timer: f32, // Timer to delay before advancing turn (for animations/pacing)
}

impl Default for TurnState {
    fn default() -> Self {
        Self {
            current_player: 0,
            action_taken: false,
            leg_number: 1,
            awaiting_action: true,
            leg_has_started: false,
            turn_delay_timer: 0.0,
        }
    }
}

/// Delay constants for different action types (in seconds)
const LEG_BET_DELAY: f32 = 0.8;
const RACE_BET_DELAY: f32 = 0.8;
const DESERT_TILE_DELAY: f32 = 1.0;
const DICE_ROLL_DELAY: f32 = 1.5; // Longer to account for animation + movement

/// Stores leg bets for each player
#[derive(Resource, Default)]
pub struct PlayerLegBetsStore {
    pub bets: Vec<Vec<LegBetTile>>, // bets[player_id] = list of tiles
}

impl PlayerLegBetsStore {
    pub fn new(player_count: usize) -> Self {
        Self {
            bets: vec![Vec::new(); player_count],
        }
    }

    pub fn add_bet(&mut self, player_id: usize, tile: LegBetTile) {
        if player_id < self.bets.len() {
            self.bets[player_id].push(tile);
        }
    }

    pub fn clear_all(&mut self) {
        for bets in &mut self.bets {
            bets.clear();
        }
    }
}

/// Stores pyramid token counts for each player
#[derive(Resource, Default)]
pub struct PlayerPyramidTokens {
    pub counts: Vec<u8>, // counts[player_id] = number of pyramid tokens
}

impl PlayerPyramidTokens {
    pub fn new(player_count: usize) -> Self {
        Self {
            counts: vec![0; player_count],
        }
    }

    pub fn add_token(&mut self, player_id: usize) {
        if player_id < self.counts.len() {
            self.counts[player_id] += 1;
        }
    }

    pub fn clear_all(&mut self) {
        for count in &mut self.counts {
            *count = 0;
        }
    }
}

/// Player action messages
#[derive(Message)]
pub struct TakeLegBetAction {
    pub color: CamelColor,
}

#[derive(Message)]
pub struct PlaceDesertTileAction {
    pub space_index: u8,
    pub is_oasis: bool,
}

#[derive(Message)]
pub struct RollPyramidAction;

#[derive(Message)]
pub struct PlaceRaceBetAction {
    pub color: CamelColor,
    pub is_winner_bet: bool,
}

/// Result of rolling the pyramid (regular camel)
#[derive(Message)]
pub struct PyramidRollResult {
    pub color: CamelColor,
    pub value: u8,
}

/// Result of rolling a crazy camel die
#[derive(Message)]
pub struct CrazyCamelRollResult {
    pub color: CrazyCamelColor,
    pub value: u8,
}

/// System to handle taking a leg bet
pub fn handle_leg_bet_action(
    mut events: MessageReader<TakeLegBetAction>,
    mut leg_tiles: ResMut<LegBettingTiles>,
    players: Res<Players>,
    mut turn_state: ResMut<TurnState>,
    mut player_leg_bets: ResMut<PlayerLegBetsStore>,
) {
    for event in events.read() {
        if turn_state.action_taken {
            continue;
        }

        if let Some(tile) = leg_tiles.take_tile(event.color) {
            let player_id = players.current_player_index;
            let player = players.current_player();
            info!("Player {} took {:?} leg bet tile worth {}", player.name, tile.camel, tile.value);
            player_leg_bets.add_bet(player_id, tile);
            turn_state.action_taken = true;
            turn_state.leg_has_started = true;
            turn_state.turn_delay_timer = LEG_BET_DELAY;
        }
    }
}

/// System to handle rolling the pyramid
pub fn handle_pyramid_roll_action(
    mut commands: Commands,
    mut events: MessageReader<RollPyramidAction>,
    mut pyramid: ResMut<Pyramid>,
    mut players: ResMut<Players>,
    mut turn_state: ResMut<TurnState>,
    mut player_pyramid_tokens: ResMut<PlayerPyramidTokens>,
    mut roll_result: MessageWriter<PyramidRollResult>,
    mut crazy_roll_result: MessageWriter<CrazyCamelRollResult>,
) {
    for _ in events.read() {
        if turn_state.action_taken {
            continue;
        }

        if let Some(die_result) = pyramid.roll_random_die() {
            let player_id = players.current_player_index;

            // Player earns 1 coin for rolling
            players.current_player_mut().money += 1;

            // Track pyramid token (only for regular dice)
            match &die_result {
                DieRollResult::Regular { .. } => {
                    player_pyramid_tokens.add_token(player_id);
                }
                DieRollResult::Crazy { .. } => {
                    // Crazy camel dice don't give pyramid tokens
                }
            }

            // Spawn animated dice sprite in center of board
            let dice_pos = Vec3::new(0.0, 0.0, 5.0); // Center, lower Z so camels render on top
            let (dice_color, roll_value) = match &die_result {
                DieRollResult::Regular { color, value } => {
                    (color.to_bevy_color(), *value)
                }
                DieRollResult::Crazy { color, value } => {
                    (color.to_bevy_color(), *value)
                }
            };

            // Spawn the dice sprite with animation and pending movement
            // Movement will be triggered when the dice animation finishes shaking
            let mut dice_entity = commands.spawn((
                DiceSprite,
                DiceRollAnimation::new(dice_pos, roll_value),
                Sprite {
                    color: dice_color,
                    custom_size: Some(Vec2::new(60.0, 60.0)),
                    ..default()
                },
                Transform::from_translation(dice_pos),
            ));

            // Add the pending movement component based on die type
            match &die_result {
                DieRollResult::Regular { color, value } => {
                    dice_entity.insert(PendingCamelMove { color: *color, spaces: *value });
                }
                DieRollResult::Crazy { color, value } => {
                    dice_entity.insert(PendingCrazyCamelMove { color: *color, spaces: *value });
                }
            }

            dice_entity.with_children(|parent| {
                // Spawn the value text as a child
                // Using a simple colored square for the pip representation
                let pip_positions = get_pip_positions(roll_value);
                for pip_pos in pip_positions {
                    parent.spawn((
                        Sprite {
                            color: Color::WHITE,
                            custom_size: Some(Vec2::new(10.0, 10.0)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(pip_pos.x, pip_pos.y, 1.0)),
                    ));
                }
            });

            // Send result events for UI updates (but NOT movement - that's triggered by animation)
            match die_result {
                DieRollResult::Regular { color, value } => {
                    info!("Rolled {:?} - {}", color, value);
                    roll_result.write(PyramidRollResult { color, value });
                }
                DieRollResult::Crazy { color, value } => {
                    info!("Rolled crazy camel {:?} - {} (moving backwards!)", color, value);
                    crazy_roll_result.write(CrazyCamelRollResult { color, value });
                }
            }

            turn_state.action_taken = true;
            turn_state.leg_has_started = true;
            turn_state.turn_delay_timer = DICE_ROLL_DELAY;
        }
    }
}

/// Get pip positions for dice display (like a real die)
fn get_pip_positions(value: u8) -> Vec<Vec2> {
    let offset = 15.0;
    match value {
        1 => vec![Vec2::ZERO],
        2 => vec![
            Vec2::new(-offset, offset),
            Vec2::new(offset, -offset),
        ],
        3 => vec![
            Vec2::new(-offset, offset),
            Vec2::ZERO,
            Vec2::new(offset, -offset),
        ],
        _ => vec![Vec2::ZERO], // Fallback
    }
}

/// System to handle race bets (overall winner/loser)
pub fn handle_race_bet_action(
    mut events: MessageReader<PlaceRaceBetAction>,
    mut race_bets: ResMut<RaceBets>,
    mut players: ResMut<Players>,
    mut turn_state: ResMut<TurnState>,
) {
    for event in events.read() {
        if turn_state.action_taken {
            continue;
        }

        let player = players.current_player_mut();

        // Check if player still has this card
        if !player.available_race_cards.contains(&event.color) {
            continue;
        }

        // Remove the card from player's hand
        player.available_race_cards.remove(&event.color);
        let player_id = player.id;
        let player_name = player.name.clone();

        if event.is_winner_bet {
            race_bets.place_winner_bet(event.color, player_id);
            info!("Player {} bet on {:?} to win", player_name, event.color);
        } else {
            race_bets.place_loser_bet(event.color, player_id);
            info!("Player {} bet on {:?} to lose", player_name, event.color);
        }

        turn_state.action_taken = true;
        turn_state.leg_has_started = true;
        turn_state.turn_delay_timer = RACE_BET_DELAY;
    }
}

/// System to handle desert tile placement
pub fn handle_desert_tile_action(
    mut events: MessageReader<PlaceDesertTileAction>,
    mut placed_tiles: ResMut<PlacedDesertTiles>,
    mut players: ResMut<Players>,
    mut turn_state: ResMut<TurnState>,
    camels: Query<&BoardPosition, With<Camel>>,
    mut commands: Commands,
    board: Res<GameBoard>,
) {
    for event in events.read() {
        if turn_state.action_taken {
            continue;
        }

        let player = players.current_player_mut();

        // Check if player has their desert tile
        if !player.has_desert_tile {
            continue;
        }

        // Check if space is valid (not space 0, no camels, no existing tiles)
        if event.space_index == 0 {
            continue;
        }

        // Check for camels on the space
        let has_camel = camels.iter().any(|pos| pos.space_index == event.space_index);
        if has_camel {
            continue;
        }

        // Check for existing tile on the space
        if placed_tiles.is_space_occupied(event.space_index) {
            continue;
        }

        // Remove any existing tile from this player
        placed_tiles.remove_player_tile(player.id);

        // Place the new tile
        placed_tiles.place_tile(event.space_index, player.id, event.is_oasis);

        // Mark player as having placed their tile (they can move it later)
        player.has_desert_tile = false;

        let tile_type = if event.is_oasis { "Oasis" } else { "Mirage" };
        info!("Player {} placed {} on space {}", player.name, tile_type, event.space_index + 1);

        // Spawn visual representation of the desert tile with polished layers
        let pos = board.get_position(event.space_index);
        let tile_size = Vec2::new(35.0, 18.0);

        let (main_color, border_color) = if event.is_oasis {
            (Color::srgb(0.3, 0.75, 0.3), Color::srgb(0.15, 0.45, 0.15)) // Green for oasis
        } else {
            (Color::srgb(0.9, 0.65, 0.35), Color::srgb(0.6, 0.4, 0.2)) // Orange/brown for mirage
        };

        // Parent entity with desert tile component
        commands.spawn((
            crate::systems::setup::GameEntity,
            DesertTile {
                owner_id: player.id,
                is_oasis: event.is_oasis,
            },
            Transform::from_xyz(pos.x, pos.y + 35.0, 5.0),
            Visibility::default(),
        )).with_children(|parent| {
            // Shadow layer
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.25),
                    custom_size: Some(tile_size),
                    ..default()
                },
                Transform::from_xyz(2.0, -2.0, -0.2),
            ));

            // Border layer
            parent.spawn((
                Sprite {
                    color: border_color,
                    custom_size: Some(tile_size + Vec2::new(3.0, 3.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));

            // Main tile
            parent.spawn((
                Sprite {
                    color: main_color,
                    custom_size: Some(tile_size),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));

            // Symbol indicator (+1 or -1)
            let symbol = if event.is_oasis { "+" } else { "-" };
            parent.spawn((
                Text2d::new(symbol.to_string()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });

        turn_state.action_taken = true;
        turn_state.leg_has_started = true;
        turn_state.turn_delay_timer = DESERT_TILE_DELAY;
    }
}

/// System to advance to the next player after an action (with delay)
pub fn advance_turn_system(
    mut turn_state: ResMut<TurnState>,
    mut players: ResMut<Players>,
    time: Res<Time>,
    ui_state: Res<crate::ui::hud::UiState>,
) {
    // Don't advance turns while leg scoring modal is showing
    if ui_state.show_leg_scoring {
        return;
    }

    if turn_state.action_taken {
        // Count down the delay timer
        if turn_state.turn_delay_timer > 0.0 {
            turn_state.turn_delay_timer -= time.delta_secs();
            return;
        }

        // Timer expired, advance to next player
        players.advance_turn();
        turn_state.current_player = players.current_player_index;
        turn_state.action_taken = false;
        turn_state.awaiting_action = true;
    }
}

/// System to check if a leg has ended (all dice rolled)
pub fn check_leg_end_system(
    pyramid: Res<Pyramid>,
    turn_state: Res<TurnState>,
    mut ui_state: ResMut<crate::ui::hud::UiState>,
) {
    // Only check for leg end if the leg has actually started (at least one action taken)
    // and we're not in the middle of processing an action (including waiting for delay timer)
    // Also check that we're not already showing the modal to avoid spamming the log
    if turn_state.leg_has_started
        && pyramid.all_dice_rolled()
        && !turn_state.action_taken
        && turn_state.turn_delay_timer <= 0.0
        && !ui_state.show_leg_scoring
    {
        info!("Leg {} complete! Showing scoring...", turn_state.leg_number);
        ui_state.show_leg_scoring = true;
    }
}

/// System to check if game has ended (camel crossed finish)
pub fn check_game_end_system(
    mut commands: Commands,
    mut events: MessageReader<MovementCompleteEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    camels: Query<(Entity, &Camel, &BoardPosition, &Transform)>,
    board: Res<GameBoard>,
) {
    for event in events.read() {
        if event.crossed_finish {
            info!("A camel crossed the finish line! Game over!");

            // Find the winning camel (highest space index, then highest stack position)
            let mut winner: Option<(Entity, u8, u8, Vec3)> = None;
            for (entity, _camel, pos, transform) in camels.iter() {
                match winner {
                    None => winner = Some((entity, pos.space_index, pos.stack_position, transform.translation)),
                    Some((_, best_space, best_stack, _)) => {
                        if pos.space_index > best_space ||
                           (pos.space_index == best_space && pos.stack_position > best_stack) {
                            winner = Some((entity, pos.space_index, pos.stack_position, transform.translation));
                        }
                    }
                }
            }

            if let Some((winner_entity, _space, _stack, current_pos)) = winner {
                // Calculate winner position: just past the finish line
                // Finish line is at space 15 (leftmost on top row)
                // Winner goes 80 pixels further left (past the finish)
                let finish_pos = board.get_position(15);
                let winner_target_x = finish_pos.x - 80.0;
                let winner_target_y = finish_pos.y; // Stay on top row Y
                let winner_target = Vec3::new(winner_target_x, winner_target_y, current_pos.z);

                // Move winner camel to victory position
                commands.entity(winner_entity).insert(
                    MovementAnimation::new(current_pos, winner_target, 0.6)
                );

                // Spawn crown above the winner (will track and land on camel)
                let crown_start = Vec3::new(winner_target_x, 400.0, 50.0);
                spawn_crown(&mut commands, crown_start, winner_entity);

                info!("Crown spawned for winning camel!");
            }

            next_state.set(GameState::GameEnd);
        }
    }
}
