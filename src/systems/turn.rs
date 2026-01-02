use bevy::prelude::*;
use bevy::input::touch::Touches;
use crate::components::*;
use crate::components::dice::DieRollResult;
use crate::components::board::{SpectatorTileSprite, PyramidRollButton, PyramidShakeAnimation, PyramidHovered};
use crate::game::state::GameState;
use crate::systems::movement::MovementCompleteEvent;
use crate::systems::animation::{DiceRollAnimation, DiceSprite, PendingCamelMove, PendingCrazyCamelMove, MovementAnimation, spawn_crown};
use crate::ui::hud::UiState;
use crate::systems::setup::PYRAMID_SIZE;

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
const SPECTATOR_TILE_DELAY: f32 = 1.0;
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
pub struct PlaceSpectatorTileAction {
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
    mut ui_state: ResMut<UiState>,
    time: Res<Time>,
) {
    for event in events.read() {
        if turn_state.action_taken {
            continue;
        }

        // Get the tile value before taking it (for animation)
        let tile_value = leg_tiles.top_tile(event.color).map(|t| t.value);

        if let Some(tile) = leg_tiles.take_tile(event.color) {
            let player_id = players.current_player_index;
            let player = players.current_player();
            info!("Player {} took {:?} leg bet tile worth {}", player.name, tile.camel, tile.value);

            // Trigger card flight animation
            // Get the card position from UiState (tracked during previous frame's render)
            let color_index = match event.color {
                CamelColor::Blue => 0,
                CamelColor::Green => 1,
                CamelColor::Red => 2,
                CamelColor::Yellow => 3,
                CamelColor::Purple => 4,
            };
            if let Some(start_pos) = ui_state.leg_bet_card_positions[color_index] {
                use crate::ui::hud::{CardFlightAnimation, CardFlightPhase};
                use bevy_egui::egui;
                // Default to top-left player area if not tracked
                let end_pos = ui_state.player_bet_area_pos.unwrap_or_else(|| egui::pos2(60.0, 40.0));
                ui_state.card_flight_animation = Some(CardFlightAnimation {
                    color: event.color,
                    value: tile_value.unwrap_or(tile.value),
                    start_pos,
                    end_pos,
                    start_time: time.elapsed_secs_f64(),
                    phase: CardFlightPhase::FlyingToPanel,
                });
            }

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
    pyramid_button: Query<Entity, With<PyramidRollButton>>,
) {
    for _ in events.read() {
        if turn_state.action_taken {
            continue;
        }

        // Calculate tent index BEFORE rolling (number of dice already rolled)
        let tent_index = pyramid.rolled_dice.len();

        if let Some(die_result) = pyramid.roll_random_die() {
            // Trigger pyramid shake animation (works for both human and AI rolls)
            if let Ok(pyramid_entity) = pyramid_button.single() {
                commands.entity(pyramid_entity).insert(PyramidShakeAnimation::new());
            }
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
            let dice_pos = Vec3::new(0.0, 0.0, 100.0); // Center, high Z to be on top
            let target_tent_pos = get_tent_world_position(tent_index);

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
            // After display, dice moves to tent and stays there
            let mut dice_entity = commands.spawn((
                DiceSprite,
                DiceRollAnimation::new(dice_pos, target_tent_pos),
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

/// Constants for tent layout (must match setup.rs)
const TENT_SPACING: f32 = 60.0;
const TENT_Y_POSITION: f32 = 200.0;

/// Calculate world position for dice in a tent
fn get_tent_world_position(tent_index: usize) -> Vec3 {
    let num_tents = 5;
    let total_width = (num_tents as f32 - 1.0) * TENT_SPACING;
    let start_x = -total_width / 2.0;
    let tent_x = start_x + (tent_index as f32 * TENT_SPACING);
    // Position dice slightly lower in tent base area
    // Z = 5.0 so dice render behind camels (which are at Z = 10-14)
    Vec3::new(tent_x, TENT_Y_POSITION - 15.0, 5.0)
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

/// System to handle spectator tile placement
pub fn handle_spectator_tile_action(
    mut events: MessageReader<PlaceSpectatorTileAction>,
    mut placed_tiles: ResMut<PlacedSpectatorTiles>,
    mut players: ResMut<Players>,
    mut turn_state: ResMut<TurnState>,
    camels: Query<&BoardPosition, With<Camel>>,
    crazy_camels: Query<&BoardPosition, With<CrazyCamel>>,
    mut commands: Commands,
    board: Res<GameBoard>,
) {
    for event in events.read() {
        if turn_state.action_taken {
            continue;
        }

        let player = players.current_player_mut();

        // Check if player has their spectator tile
        if !player.has_spectator_tile {
            continue;
        }

        // Check if space is valid (not space 0, no camels, no existing tiles)
        if event.space_index == 0 {
            continue;
        }

        // Check for camels on the space (including crazy camels)
        let has_camel = camels.iter().any(|pos| pos.space_index == event.space_index);
        let has_crazy_camel = crazy_camels.iter().any(|pos| pos.space_index == event.space_index);
        if has_camel || has_crazy_camel {
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
        player.has_spectator_tile = false;

        let tile_type = if event.is_oasis { "Oasis" } else { "Mirage" };
        info!("Player {} placed {} on space {}", player.name, tile_type, event.space_index + 1);

        // Spawn visual representation of the spectator tile with polished layers
        let pos = board.get_position(event.space_index);
        let tile_size = Vec2::new(35.0, 18.0);

        let (main_color, border_color) = if event.is_oasis {
            (Color::srgb(0.3, 0.75, 0.3), Color::srgb(0.15, 0.45, 0.15)) // Green for oasis
        } else {
            (Color::srgb(0.9, 0.65, 0.35), Color::srgb(0.6, 0.4, 0.2)) // Orange/brown for mirage
        };

        // Parent entity with spectator tile component
        commands.spawn((
            crate::systems::setup::GameEntity,
            SpectatorTile {
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
        turn_state.turn_delay_timer = SPECTATOR_TILE_DELAY;
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

                // Spawn crown directly on the winner's head
                // Winner faces LEFT (toward finish), so head is on the left side
                // Head is at (-22, +16) relative to camel center when facing left (14px wide, 10px tall)
                // Crown is 18px wide, head is 14px wide
                let head_center_x = winner_target_x - 22.0;
                let head_top_y = winner_target_y + 16.0 + 5.0; // head height is 10, so +5 to top
                // Shift crown 2px right so it doesn't extend as far to the left
                let crown_pos = Vec3::new(head_center_x + 2.0, head_top_y + 8.0, 50.0);
                spawn_crown(&mut commands, crown_pos, Some(150.0));

                info!("Crown spawned for winning camel!");
            }

            next_state.set(GameState::GameEnd);
        }
    }
}

/// System to update spectator tile sprite colors based on game state
/// Each board space has a spectator tile sprite that can be:
/// - Transparent (invisible): no tile selection active and no tile placed
/// - Semi-transparent green/orange: tile card selected and this is a valid placement space
/// - Fully opaque: tile has been placed on this space
pub fn update_spectator_tile_sprites(
    ui_state: Res<UiState>,
    players: Option<Res<Players>>,
    camels: Query<&BoardPosition, With<Camel>>,
    crazy_camels: Query<&BoardPosition, With<CrazyCamel>>,
    placed_tiles: Option<Res<PlacedSpectatorTiles>>,
    turn_state: Option<Res<TurnState>>,
    mut tile_sprites: Query<(&SpectatorTileSprite, &mut Sprite, &Children)>,
    mut text_query: Query<&mut TextColor>,
) {
    let Some(players) = players else { return };
    let Some(placed_tiles) = placed_tiles else { return };
    let Some(turn_state) = turn_state else { return };

    let current = players.current_player();
    let is_selecting = ui_state.spectator_tile_selected
        && current.has_spectator_tile
        && !turn_state.action_taken
        && !current.is_ai
        && ui_state.initial_rolls_complete;

    for (tile_sprite, mut sprite, children) in tile_sprites.iter_mut() {
        let space = tile_sprite.space_index;

        // Check if a tile is placed on this space
        if let Some((_owner_id, is_oasis)) = placed_tiles.get_tile(space) {
            // Tile is placed here - show fully opaque
            let color = if is_oasis {
                Color::srgb(0.3, 0.75, 0.3) // Green for oasis
            } else {
                Color::srgb(0.9, 0.65, 0.35) // Orange for mirage
            };
            sprite.color = color;

            // Update text color to white
            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    text_color.0 = Color::WHITE;
                }
            }
            continue;
        }

        // No tile placed - check if we should show preview
        if !is_selecting || space == 0 {
            // Not selecting or space 0 (can't place there) - hide the tile
            sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    text_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
                }
            }
            continue;
        }

        // Check if this space is valid for placement (including crazy camels)
        let has_camel = camels.iter().any(|pos| pos.space_index == space);
        let has_crazy_camel = crazy_camels.iter().any(|pos| pos.space_index == space);
        let has_other_tile = placed_tiles.tiles.iter()
            .any(|(&s, &(owner, _))| s == space && owner != current.id);

        if has_camel || has_crazy_camel || has_other_tile {
            // Invalid space - hide or show as invalid (red tint)
            sprite.color = Color::srgba(0.5, 0.2, 0.2, 0.3); // Semi-transparent red
            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    text_color.0 = Color::srgba(1.0, 0.5, 0.5, 0.3);
                }
            }
        } else {
            // Valid space - show semi-transparent preview
            let (color, text_alpha) = if ui_state.spectator_tile_is_oasis {
                (Color::srgba(0.3, 0.75, 0.3, 0.5), 0.7) // Semi-transparent green
            } else {
                (Color::srgba(0.9, 0.65, 0.35, 0.5), 0.7) // Semi-transparent orange
            };
            sprite.color = color;

            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    text_color.0 = Color::srgba(1.0, 1.0, 1.0, text_alpha);
                }
            }
        }
    }
}

/// System to handle clicks/taps on spectator tile sprites for placement
pub fn handle_spectator_tile_clicks(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    tile_sprites: Query<(&GlobalTransform, &SpectatorTileSprite, &Sprite)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut ui_state: ResMut<UiState>,
    mut action: MessageWriter<PlaceSpectatorTileAction>,
    players: Option<Res<Players>>,
    camels: Query<&BoardPosition, With<Camel>>,
    crazy_camels: Query<&BoardPosition, With<CrazyCamel>>,
    placed_tiles: Option<Res<PlacedSpectatorTiles>>,
) {
    // Only process if tile is selected
    if !ui_state.spectator_tile_selected {
        return;
    }

    let Some(players) = players else { return };
    let Some(placed_tiles) = placed_tiles else { return };

    // Get click/tap position
    let click_pos = if mouse_input.just_pressed(MouseButton::Left) {
        windows.single().ok().and_then(|w| w.cursor_position())
    } else if let Some(touch) = touches.iter_just_pressed().next() {
        Some(touch.position())
    } else {
        None
    };

    let Some(screen_pos) = click_pos else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };

    // Convert screen position to world position
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) else { return };

    // Check if click is on any tile sprite
    let tile_size = Vec2::new(35.0, 18.0);
    let half_size = tile_size * 0.5;
    let current = players.current_player();

    for (transform, tile_sprite, sprite) in tile_sprites.iter() {
        // Skip invisible tiles (alpha near 0)
        if sprite.color.alpha() < 0.1 {
            continue;
        }

        let space = tile_sprite.space_index;

        // Skip space 0 (can't place there)
        if space == 0 {
            continue;
        }

        // Check if this is a valid space (including crazy camels)
        let has_camel = camels.iter().any(|pos| pos.space_index == space);
        let has_crazy_camel = crazy_camels.iter().any(|pos| pos.space_index == space);
        let has_other_tile = placed_tiles.tiles.iter()
            .any(|(&s, &(owner, _))| s == space && owner != current.id);

        if has_camel || has_crazy_camel || has_other_tile {
            continue; // Can't place on invalid spaces
        }

        let tile_pos = transform.translation().truncate();
        let min = tile_pos - half_size;
        let max = tile_pos + half_size;

        if world_pos.x >= min.x && world_pos.x <= max.x
            && world_pos.y >= min.y && world_pos.y <= max.y
        {
            // Clicked on this valid space - place the tile!
            action.write(PlaceSpectatorTileAction {
                space_index: space,
                is_oasis: ui_state.spectator_tile_is_oasis,
            });

            // Deselect the tile card
            ui_state.spectator_tile_selected = false;
            return;
        }
    }
}

/// System to handle clicks/taps on the pyramid roll button sprite
pub fn handle_pyramid_click(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    pyramid_query: Query<&GlobalTransform, With<PyramidRollButton>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    ui_state: Res<UiState>,
    mut roll_action: MessageWriter<RollPyramidAction>,
    players: Option<Res<Players>>,
    turn_state: Res<TurnState>,
    pyramid: Res<Pyramid>,
    shake_query: Query<(), With<PyramidShakeAnimation>>,
) {
    // Don't process if initial rolls aren't complete
    if !ui_state.initial_rolls_complete {
        return;
    }

    // Don't process if showing leg scoring modal
    if ui_state.show_leg_scoring {
        return;
    }

    // Don't process if already shaking
    if !shake_query.is_empty() {
        return;
    }

    let Some(players) = players else { return };
    let current = players.current_player();

    // Only allow human players to click
    if current.is_ai {
        return;
    }

    // Don't allow action if turn already taken
    if turn_state.action_taken {
        return;
    }

    // Check if all dice have been rolled (can't roll pyramid if leg is complete)
    if pyramid.all_dice_rolled() {
        return;
    }

    // Get click/tap position
    let click_pos = if mouse_input.just_pressed(MouseButton::Left) {
        windows.single().ok().and_then(|w| w.cursor_position())
    } else if let Some(touch) = touches.iter_just_pressed().next() {
        Some(touch.position())
    } else {
        None
    };

    let Some(screen_pos) = click_pos else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };

    // Convert screen position to world position
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) else { return };

    // Check if click is on pyramid
    let pyramid_size = Vec2::splat(PYRAMID_SIZE);
    let half_size = pyramid_size * 0.5;

    for transform in pyramid_query.iter() {
        let pyramid_pos = transform.translation().truncate();
        let min = pyramid_pos - half_size;
        let max = pyramid_pos + half_size;

        if world_pos.x >= min.x && world_pos.x <= max.x
            && world_pos.y >= min.y && world_pos.y <= max.y
        {
            // Clicked on pyramid - trigger roll!
            // Shake animation is triggered in handle_pyramid_roll_action
            roll_action.write(RollPyramidAction);
            return;
        }
    }
}

/// System to detect hover over the pyramid roll button
pub fn handle_pyramid_hover(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    pyramid_query: Query<(Entity, &GlobalTransform, Option<&PyramidHovered>), With<PyramidRollButton>>,
    mut commands: Commands,
    ui_state: Res<UiState>,
    players: Option<Res<Players>>,
    turn_state: Res<TurnState>,
    pyramid: Res<Pyramid>,
    shake_query: Query<(), With<PyramidShakeAnimation>>,
) {
    // Get cursor position
    let cursor_pos = windows.single().ok().and_then(|w| w.cursor_position());
    let Some(screen_pos) = cursor_pos else {
        // No cursor - remove hover from all pyramids
        for (entity, _, hovered) in pyramid_query.iter() {
            if hovered.is_some() {
                commands.entity(entity).remove::<PyramidHovered>();
            }
        }
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) else { return };

    // Check if pyramid is interactive (can be clicked)
    let is_interactive = ui_state.initial_rolls_complete
        && !ui_state.show_leg_scoring
        && shake_query.is_empty()
        && players.as_ref().map_or(false, |p| !p.current_player().is_ai)
        && !turn_state.action_taken
        && !pyramid.all_dice_rolled();

    let pyramid_size = Vec2::splat(PYRAMID_SIZE);
    let half_size = pyramid_size * 0.5;

    for (entity, transform, hovered) in pyramid_query.iter() {
        let pyramid_pos = transform.translation().truncate();
        let min = pyramid_pos - half_size;
        let max = pyramid_pos + half_size;

        let is_over = world_pos.x >= min.x && world_pos.x <= max.x
            && world_pos.y >= min.y && world_pos.y <= max.y;

        if is_over && is_interactive {
            if hovered.is_none() {
                commands.entity(entity).insert(PyramidHovered);
            }
        } else {
            if hovered.is_some() {
                commands.entity(entity).remove::<PyramidHovered>();
            }
        }
    }
}
