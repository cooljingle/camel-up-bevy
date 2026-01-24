//! Lobby and Waiting Room UI for online multiplayer

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::game::state::GameState;
use crate::network::state::{NetworkState, RoomPlayers};

#[cfg(target_arch = "wasm32")]
use crate::network::state::{NetworkMode, OnlinePlayerInfo};
use crate::network::room::generate_room_code;
use crate::ui::hud::UiState;
use crate::ui::theme::{desert_button, DesertButtonStyle, STONE_DARK, PLAYER_COLORS};
use crate::ui::characters::{draw_avatar, CharacterId};
use crate::ui::player_setup::PlayerSetupConfig;

#[cfg(target_arch = "wasm32")]
use crate::network::js_bindings;

/// State for the lobby UI
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct LobbyState {
    pub screen: LobbyScreen,
    pub room_code_input: String,
    pub player_name: String,
    pub selected_character: CharacterId,
    pub selected_color: usize,
    pub error_message: Option<String>,
    pub is_loading: bool,
    pub is_ready: bool,
    pub firebase_initialized: bool,
    pub firebase_user_id: Option<String>,
    pub appearance_initialized: bool,  // Track if we've set up unique appearance in waiting room
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum LobbyScreen {
    #[default]
    Main,       // Choose create or join
    Create,     // Creating a room
    Join,       // Entering room code
}

/// Colors for the desert theme (from main_menu.rs)
const SKY_BLUE: egui::Color32 = egui::Color32::from_rgb(0x87, 0xCE, 0xEB);
const SAND_COLOR: egui::Color32 = egui::Color32::from_rgb(0xED, 0xC9, 0x9A);
const PYRAMID_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);
const PYRAMID_DARK: egui::Color32 = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);

/// Draw a simple desert background
fn draw_desert_background(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0.0, SKY_BLUE);
    let horizon_y = rect.top() + rect.height() * 0.75;
    let sand_rect = egui::Rect::from_min_max(egui::pos2(rect.left(), horizon_y), rect.max);
    painter.rect_filled(sand_rect, 0.0, SAND_COLOR);

    // Simple pyramid silhouette
    let pyramid_width = rect.width() * 0.4;
    let pyramid_height = pyramid_width * 0.5;
    let center_x = rect.center().x;
    let apex = egui::pos2(center_x, horizon_y - pyramid_height);
    let base_left = egui::pos2(center_x - pyramid_width / 2.0, horizon_y);
    let base_right = egui::pos2(center_x + pyramid_width / 2.0, horizon_y);

    painter.add(egui::Shape::convex_polygon(
        vec![apex, base_left, egui::pos2(center_x, horizon_y)],
        PYRAMID_DARK,
        egui::Stroke::NONE,
    ));
    painter.add(egui::Shape::convex_polygon(
        vec![apex, egui::pos2(center_x, horizon_y), base_right],
        PYRAMID_LIGHT,
        egui::Stroke::NONE,
    ));
}

/// Lobby UI system - create or join a room
pub fn lobby_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut lobby_state: ResMut<LobbyState>,
    mut network_state: ResMut<NetworkState>,
    ui_state: Res<UiState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let is_mobile = !ui_state.use_side_panels;

    // Initialize Firebase on first frame (WASM only)
    #[cfg(target_arch = "wasm32")]
    if !lobby_state.firebase_initialized {
        lobby_state.firebase_initialized = true;
        lobby_state.is_loading = true;

        // Initialize Firebase and sign in
        js_bindings::async_ops::init_and_authenticate(|result| {
            // This callback runs asynchronously - we'll poll for completion
        });
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Check if authentication completed
        if lobby_state.is_loading && lobby_state.firebase_user_id.is_none() {
            if let Some(uid) = js_bindings::get_current_user_id() {
                lobby_state.firebase_user_id = Some(uid.clone());
                lobby_state.is_loading = false;
                network_state.local_player_id = Some(uid);
            } else if let Some(err) = js_bindings::get_firebase_error() {
                lobby_state.error_message = Some(format!("Firebase error: {}", err));
                lobby_state.is_loading = false;
            }
        }
    }

    // For non-WASM, just mark as ready (multiplayer won't work but UI will render)
    #[cfg(not(target_arch = "wasm32"))]
    {
        if !lobby_state.firebase_initialized {
            lobby_state.firebase_initialized = true;
            lobby_state.error_message = Some("Multiplayer only available in browser".to_string());
        }
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            draw_desert_background(ui.painter(), rect);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(if is_mobile { 30.0 } else { 50.0 });

                    // Title
                    ui.heading(
                        egui::RichText::new("PLAY ONLINE")
                            .size(if is_mobile { 28.0 } else { 40.0 })
                            .color(egui::Color32::WHITE),
                    );

                    // Version number for debugging
                    ui.label(
                        egui::RichText::new("v0.1.7")
                            .size(12.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 120)),
                    );
                    ui.add_space(10.0);

                    // Show loading state
                    if lobby_state.is_loading {
                        ui.add_space(40.0);
                        ui.label(
                            egui::RichText::new("Connecting...")
                                .size(18.0)
                                .color(egui::Color32::WHITE),
                        );
                        ui.add_space(10.0);
                        ui.spinner();
                        return;
                    }

                    // Show error if any
                    if let Some(ref error) = lobby_state.error_message {
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(error)
                                .size(14.0)
                                .color(egui::Color32::from_rgb(255, 100, 100)),
                        );
                        ui.add_space(10.0);
                    }

                    ui.add_space(20.0);

                    match lobby_state.screen {
                        LobbyScreen::Main => {
                            draw_main_lobby_screen(ui, &mut lobby_state, &mut next_state, is_mobile);
                        }
                        LobbyScreen::Create => {
                            draw_create_room_screen(
                                ui,
                                &mut lobby_state,
                                &mut network_state,
                                &mut next_state,
                                is_mobile,
                            );
                        }
                        LobbyScreen::Join => {
                            draw_join_room_screen(
                                ui,
                                &mut lobby_state,
                                &mut network_state,
                                &mut next_state,
                                is_mobile,
                            );
                        }
                    }
                });
            });
        });
}

fn draw_main_lobby_screen(
    ui: &mut egui::Ui,
    lobby_state: &mut LobbyState,
    next_state: &mut NextState<GameState>,
    is_mobile: bool,
) {
    let button_style = if is_mobile {
        DesertButtonStyle {
            min_size: egui::vec2(260.0, 55.0),
            corner_radius: 10.0,
            font_size: 20.0,
        }
    } else {
        DesertButtonStyle::large()
    };

    ui.add_space(20.0);

    if desert_button(ui, "Create Room", &button_style).clicked() {
        lobby_state.screen = LobbyScreen::Create;
        lobby_state.error_message = None;
        // Generate a random room code
        lobby_state.room_code_input = generate_room_code();
    }

    ui.add_space(15.0);

    if desert_button(ui, "Join Room", &button_style).clicked() {
        lobby_state.screen = LobbyScreen::Join;
        lobby_state.error_message = None;
        lobby_state.room_code_input.clear();
    }

    ui.add_space(30.0);

    let back_style = DesertButtonStyle::small();
    if desert_button(ui, "Back to Menu", &back_style).clicked() {
        next_state.set(GameState::MainMenu);
    }
}

#[allow(unused_variables)]
fn draw_create_room_screen(
    ui: &mut egui::Ui,
    lobby_state: &mut LobbyState,
    network_state: &mut NetworkState,
    next_state: &mut NextState<GameState>,
    is_mobile: bool,
) {
    ui.label(
        egui::RichText::new("Create a Room")
            .size(22.0)
            .color(egui::Color32::WHITE),
    );

    ui.add_space(20.0);

    // Room code display
    ui.label(egui::RichText::new("Room Code:").color(egui::Color32::WHITE));
    ui.add_space(5.0);
    ui.label(
        egui::RichText::new(&lobby_state.room_code_input)
            .size(32.0)
            .color(egui::Color32::from_rgb(255, 215, 0))
            .monospace(),
    );
    ui.add_space(5.0);
    ui.label(
        egui::RichText::new("Share this code with friends!")
            .size(12.0)
            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180)),
    );

    ui.add_space(25.0);

    let button_style = DesertButtonStyle::medium();

    // Create button
    let can_create = !lobby_state.is_loading;

    if desert_button(ui, "Create & Wait for Players", &button_style).clicked() && can_create {
        #[cfg(target_arch = "wasm32")]
        {
            lobby_state.is_loading = true;
            let room_code = lobby_state.room_code_input.clone();
            // Generate a random thematic name for online players
            let player_name = lobby_state.selected_character.random_name();
            lobby_state.player_name = player_name.clone();
            let character_id = lobby_state.selected_character as u8;
            let color_index = lobby_state.selected_color;

            js_bindings::async_ops::create_room_async(
                room_code.clone(),
                player_name,
                character_id,
                color_index,
                move |result| {
                    // Result will be checked by polling
                },
            );

            // Set network state
            network_state.mode = NetworkMode::OnlineHost;
            network_state.room_code = Some(room_code);
            network_state.is_connected = true;

            // Go to waiting room
            next_state.set(GameState::WaitingRoom);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            lobby_state.error_message = Some("Multiplayer only available in browser".to_string());
        }
    }

    ui.add_space(15.0);

    // Back button
    let back_style = DesertButtonStyle::small();
    if desert_button(ui, "Back", &back_style).clicked() {
        lobby_state.screen = LobbyScreen::Main;
        lobby_state.error_message = None;
    }
}

#[allow(unused_variables)]
fn draw_join_room_screen(
    ui: &mut egui::Ui,
    lobby_state: &mut LobbyState,
    network_state: &mut NetworkState,
    next_state: &mut NextState<GameState>,
    is_mobile: bool,
) {
    ui.label(
        egui::RichText::new("Join a Room")
            .size(22.0)
            .color(egui::Color32::WHITE),
    );

    ui.add_space(20.0);

    // Room code input
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Room Code:").color(egui::Color32::WHITE));
        ui.add_space(8.0);
        let text_edit = egui::TextEdit::singleline(&mut lobby_state.room_code_input)
            .desired_width(120.0)
            .char_limit(4)
            .font(egui::FontId::monospace(20.0))
            .text_color(egui::Color32::from_rgb(255, 215, 0));
        ui.scope(|ui| {
            ui.visuals_mut().extreme_bg_color = STONE_DARK;
            ui.add(text_edit);
        });
    });

    // Auto-uppercase the room code
    lobby_state.room_code_input = lobby_state.room_code_input.to_uppercase();

    ui.add_space(25.0);

    let button_style = DesertButtonStyle::medium();

    // Join button
    let can_join = lobby_state.room_code_input.len() == 4
        && !lobby_state.is_loading;

    if desert_button(ui, "Join Room", &button_style).clicked() && can_join {
        #[cfg(target_arch = "wasm32")]
        {
            lobby_state.is_loading = true;
            let room_code = lobby_state.room_code_input.clone();
            // Generate a random thematic name for online players
            let player_name = lobby_state.selected_character.random_name();
            lobby_state.player_name = player_name.clone();
            let character_id = lobby_state.selected_character as u8;
            let color_index = lobby_state.selected_color;

            js_bindings::async_ops::join_room_async(
                room_code.clone(),
                player_name,
                character_id,
                color_index,
                move |result| {
                    // Result will be checked by polling
                },
            );

            // Set network state
            network_state.mode = NetworkMode::OnlineClient;
            network_state.room_code = Some(room_code);
            network_state.is_connected = true;

            // Go to waiting room
            next_state.set(GameState::WaitingRoom);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            lobby_state.error_message = Some("Multiplayer only available in browser".to_string());
        }
    }

    ui.add_space(15.0);

    // Back button
    let back_style = DesertButtonStyle::small();
    if desert_button(ui, "Back", &back_style).clicked() {
        lobby_state.screen = LobbyScreen::Main;
        lobby_state.error_message = None;
    }
}

/// Waiting Room UI - wait for players before game starts
#[allow(unused_variables, unused_mut)]
pub fn waiting_room_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut lobby_state: ResMut<LobbyState>,
    mut network_state: ResMut<NetworkState>,
    mut room_players: ResMut<RoomPlayers>,
    mut config: ResMut<PlayerSetupConfig>,
    ui_state: Res<UiState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let is_mobile = !ui_state.use_side_panels;
    let is_host = network_state.is_host();

    // Subscribe to player updates (WASM only)
    #[cfg(target_arch = "wasm32")]
    {
        static mut SUBSCRIBED: bool = false;
        unsafe {
            if !SUBSCRIBED {
                if let Some(ref room_code) = network_state.room_code {
                    js_bindings::subscribe_to_players(room_code);
                    js_bindings::subscribe_to_metadata(room_code);
                    SUBSCRIBED = true;
                }
            }
        }

        // Poll for player updates
        if let Some(players_json) = js_bindings::poll_players() {
            if let Ok(players) = serde_json::from_str::<Vec<OnlinePlayerInfo>>(&players_json) {
                room_players.players = players;
            }
        }

        // Auto-select unique character/color when first entering waiting room
        if !lobby_state.appearance_initialized && !room_players.players.is_empty() {
            lobby_state.appearance_initialized = true;

            // Find our current player info
            let my_id = network_state.local_player_id.as_ref();

            // Collect taken characters and colors from OTHER players
            let taken_characters: std::collections::HashSet<u8> = room_players.players.iter()
                .filter(|p| Some(&p.id) != my_id)
                .map(|p| p.character_id)
                .collect();
            let taken_colors: std::collections::HashSet<usize> = room_players.players.iter()
                .filter(|p| Some(&p.id) != my_id)
                .map(|p| p.color_index)
                .collect();

            // Find first available character (0-15)
            let available_char = (0u8..16).find(|c| !taken_characters.contains(c)).unwrap_or(0);
            // Find first available color (0-7)
            let available_color = (0usize..8).find(|c| !taken_colors.contains(c)).unwrap_or(0);

            // Check if we need to update (if different from what we joined with)
            let needs_update = lobby_state.selected_character as u8 != available_char
                || lobby_state.selected_color != available_color;

            if needs_update {
                lobby_state.selected_character = CharacterId::from_index(available_char as usize);
                lobby_state.selected_color = available_color;

                // Update Firebase
                if let Some(ref room_code) = network_state.room_code {
                    js_bindings::async_ops::update_appearance_async(
                        room_code.clone(),
                        available_char,
                        available_color,
                        None,
                    );
                }
            }
        }

        // Check if game has started (for clients)
        if !is_host && js_bindings::has_game_started() {
            // Subscribe to game state updates
            if let Some(ref room_code) = network_state.room_code {
                js_bindings::subscribe_to_game_state(room_code);
            }
            // Set up local players based on room players
            setup_players_from_room(&room_players, &network_state, &mut config);
            next_state.set(GameState::Playing);
        }
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            draw_desert_background(ui.painter(), rect);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(if is_mobile { 30.0 } else { 50.0 });

                    // Title
                    ui.heading(
                        egui::RichText::new("WAITING ROOM")
                            .size(if is_mobile { 24.0 } else { 32.0 })
                            .color(egui::Color32::WHITE),
                    );

                    ui.add_space(15.0);

                    // Room code display
                    if let Some(ref code) = network_state.room_code {
                        ui.label(
                            egui::RichText::new(format!("Room Code: {}", code))
                                .size(24.0)
                                .color(egui::Color32::from_rgb(255, 215, 0))
                                .monospace(),
                        );
                    }

                    ui.add_space(10.0);

                    if is_host {
                        ui.label(
                            egui::RichText::new("Share this code with friends to join!")
                                .size(14.0)
                                .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180)),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("Waiting for host to start the game...")
                                .size(14.0)
                                .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180)),
                        );
                    }

                    ui.add_space(25.0);

                    // Player list
                    ui.label(
                        egui::RichText::new(format!("Players ({}/8)", room_players.players.len()))
                            .size(18.0)
                            .color(egui::Color32::WHITE),
                    );

                    ui.add_space(10.0);

                    #[cfg(target_arch = "wasm32")]
                    {
                        // Collect taken characters and colors from OTHER players (for cycling)
                        let my_id = network_state.local_player_id.as_ref();
                        let taken_characters: std::collections::HashSet<u8> = room_players.players.iter()
                            .filter(|p| Some(&p.id) != my_id)
                            .map(|p| p.character_id)
                            .collect();
                        let taken_colors: std::collections::HashSet<usize> = room_players.players.iter()
                            .filter(|p| Some(&p.id) != my_id)
                            .map(|p| p.color_index)
                            .collect();

                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100))
                            .inner_margin(15.0)
                            .corner_radius(8.0)
                            .show(ui, |ui| {
                                if room_players.players.is_empty() {
                                    ui.label(
                                        egui::RichText::new("No players yet...")
                                            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150)),
                                    );
                                } else {
                                    for player in &room_players.players {
                                        let is_local_player = Some(&player.id) == my_id;
                                        let player_color = PLAYER_COLORS[player.color_index % PLAYER_COLORS.len()];

                                        // Use fixed-height row with centered vertical alignment (like start screen)
                                        let row_height = 44.0;
                                        ui.allocate_ui(
                                            egui::vec2(ui.available_width(), row_height),
                                            |ui| {
                                                ui.with_layout(
                                                    egui::Layout::left_to_right(egui::Align::Center),
                                                    |ui| {
                                                        ui.add_space(10.0);

                                                        // Avatar - clickable only for local player
                                                        let avatar_size = 40.0;
                                                        let sense = if is_local_player {
                                                            egui::Sense::click()
                                                        } else {
                                                            egui::Sense::hover()
                                                        };
                                                        let (rect, response) = ui.allocate_exact_size(
                                                            egui::vec2(avatar_size, avatar_size),
                                                            sense,
                                                        );

                                                        draw_avatar(
                                                            ui.painter(),
                                                            rect,
                                                            CharacterId::from_index(player.character_id as usize),
                                                            Some(player_color),
                                                        );

                                                        // Cycle both character AND color on click (local player only)
                                                        if is_local_player && response.clicked() {
                                                            // Find next available character
                                                            let current_char = lobby_state.selected_character as u8;
                                                            let next_char = ((current_char + 1)..16).chain(0..current_char)
                                                                .find(|c| !taken_characters.contains(c))
                                                                .unwrap_or(current_char);

                                                            // Find next available color
                                                            let current_color = lobby_state.selected_color;
                                                            let next_color = ((current_color + 1)..8).chain(0..current_color)
                                                                .find(|c| !taken_colors.contains(c))
                                                                .unwrap_or(current_color);

                                                            lobby_state.selected_character = CharacterId::from_index(next_char as usize);
                                                            lobby_state.selected_color = next_color;

                                                            // Update Firebase
                                                            if let Some(ref room_code) = network_state.room_code {
                                                                js_bindings::async_ops::update_appearance_async(
                                                                    room_code.clone(),
                                                                    next_char,
                                                                    next_color,
                                                                    None,
                                                                );
                                                            }
                                                        }

                                                        ui.add_space(10.0);

                                                        // Name - editable for local player, label for others
                                                        let name_width = 120.0;
                                                        if is_local_player {
                                                            // Editable name input
                                                            let text_edit = egui::TextEdit::singleline(&mut lobby_state.player_name)
                                                                .desired_width(name_width)
                                                                .font(egui::FontId::proportional(14.0))
                                                                .text_color(egui::Color32::WHITE);
                                                            let response = ui.scope(|ui| {
                                                                ui.visuals_mut().extreme_bg_color = STONE_DARK;
                                                                ui.add(text_edit)
                                                            }).inner;

                                                            // Update Firebase when name changes
                                                            if response.changed() {
                                                                if let Some(ref room_code) = network_state.room_code {
                                                                    js_bindings::async_ops::update_appearance_async(
                                                                        room_code.clone(),
                                                                        lobby_state.selected_character as u8,
                                                                        lobby_state.selected_color,
                                                                        Some(lobby_state.player_name.clone()),
                                                                    );
                                                                }
                                                            }
                                                        } else {
                                                            // Read-only name label
                                                            ui.label(
                                                                egui::RichText::new(&player.name)
                                                                    .size(14.0)
                                                                    .color(egui::Color32::WHITE),
                                                            );
                                                        }

                                                        ui.add_space(8.0);

                                                        // Host indicator
                                                        if player.is_host {
                                                            ui.label(
                                                                egui::RichText::new("(Host)")
                                                                    .size(12.0)
                                                                    .color(egui::Color32::from_rgb(255, 215, 0)),
                                                            );
                                                        }

                                                        ui.add_space(10.0);
                                                    },
                                                );
                                            },
                                        );
                                        ui.add_space(2.0);
                                    }
                                }
                            });
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100))
                            .inner_margin(15.0)
                            .corner_radius(8.0)
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("Online multiplayer not available in native builds")
                                        .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150)),
                                );
                            });
                    }

                    ui.add_space(20.0);

                    // Randomize order toggle - visible to all, editable by host only
                    #[cfg(target_arch = "wasm32")]
                    {
                        let randomize_order = js_bindings::get_randomize_order();

                        ui.horizontal(|ui| {
                            // Draw checkbox manually to show correct state
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(20.0, 20.0),
                                if is_host { egui::Sense::click() } else { egui::Sense::hover() },
                            );

                            // Draw checkbox background
                            ui.painter().rect_filled(
                                rect,
                                3.0,
                                if is_host { STONE_DARK } else { egui::Color32::from_rgba_unmultiplied(60, 60, 60, 200) },
                            );
                            ui.painter().rect_stroke(
                                rect,
                                3.0,
                                egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)),
                                egui::epaint::StrokeKind::Outside,
                            );

                            // Draw checkmark if enabled
                            if randomize_order {
                                let check_color = egui::Color32::from_rgb(100, 255, 100);
                                let center = rect.center();
                                let size = rect.width() * 0.3;
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(center.x - size, center.y),
                                        egui::pos2(center.x - size * 0.3, center.y + size * 0.7),
                                    ],
                                    egui::Stroke::new(2.5, check_color),
                                );
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(center.x - size * 0.3, center.y + size * 0.7),
                                        egui::pos2(center.x + size, center.y - size * 0.5),
                                    ],
                                    egui::Stroke::new(2.5, check_color),
                                );
                            }

                            // Handle click (host only)
                            if is_host && response.clicked() {
                                if let Some(ref room_code) = network_state.room_code {
                                    js_bindings::async_ops::set_randomize_order_async(
                                        room_code.clone(),
                                        !randomize_order,
                                    );
                                }
                            }

                            ui.add_space(8.0);

                            let label_color = if is_host {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150)
                            };
                            ui.label(
                                egui::RichText::new("Randomize play order")
                                    .size(14.0)
                                    .color(label_color),
                            );

                            if !is_host {
                                ui.label(
                                    egui::RichText::new("(host only)")
                                        .size(11.0)
                                        .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)),
                                );
                            }
                        });
                    }

                    ui.add_space(15.0);

                    // Host controls
                    if is_host {
                        let button_style = DesertButtonStyle::large();
                        let can_start = room_players.players.len() >= 2;

                        if desert_button(ui, "Start Game", &button_style).clicked() && can_start {
                            #[cfg(target_arch = "wasm32")]
                            {
                                if let Some(ref room_code) = network_state.room_code {
                                    // Subscribe to actions from clients
                                    js_bindings::subscribe_to_actions(room_code);

                                    js_bindings::async_ops::start_game_async(
                                        room_code.clone(),
                                        |_| {},
                                    );
                                }

                                // Set up local players based on room players
                                setup_players_from_room(&room_players, &network_state, &mut config);
                            }
                            next_state.set(GameState::Playing);
                        }

                        if !can_start {
                            ui.add_space(5.0);
                            ui.label(
                                egui::RichText::new("Need at least 2 players to start")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150)),
                            );
                        }
                    }

                    ui.add_space(20.0);

                    // Leave room button
                    let back_style = DesertButtonStyle::small();
                    if desert_button(ui, "Leave Room", &back_style).clicked() {
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(ref room_code) = network_state.room_code {
                                js_bindings::async_ops::leave_room_async(room_code.clone());
                                js_bindings::unsubscribe_all();
                            }
                            // Reset subscribed flag
                            unsafe {
                                static mut SUBSCRIBED: bool = false;
                                SUBSCRIBED = false;
                            }
                        }
                        lobby_state.appearance_initialized = false;
                        network_state.reset();
                        next_state.set(GameState::Lobby);
                    }
                });
            });
        });
}

/// Set up the player configuration from room players
#[allow(unused_variables, dead_code)]
fn setup_players_from_room(
    room_players: &RoomPlayers,
    network_state: &NetworkState,
    config: &mut PlayerSetupConfig,
) {
    use rand::seq::SliceRandom;

    config.players.clear();

    // Get the list of players and sort so host is always first
    let mut players: Vec<_> = room_players.players.iter().collect();
    players.sort_by(|a, b| b.is_host.cmp(&a.is_host)); // Host (true) comes before non-host (false)

    // Optionally randomize order based on Firebase setting (but keep host first)
    #[cfg(target_arch = "wasm32")]
    {
        if js_bindings::get_randomize_order() {
            let mut rng = rand::thread_rng();
            // Only shuffle non-host players (skip first player which is host)
            if players.len() > 1 {
                players[1..].shuffle(&mut rng);
            }
        }
    }

    for player in players {
        let _is_local = Some(&player.id) == network_state.local_player_id.as_ref();

        config.players.push(crate::ui::player_setup::PlayerConfig {
            name: player.name.clone(),
            is_ai: false, // Online players are never AI
            character_id: CharacterId::from_index(player.character_id as usize),
            color_index: player.color_index,
            name_edited: true,
        });

        // Track which player index is the local player
        // (Currently unused but will be needed for turn management)
    }
}

/// Cleanup when leaving lobby/waiting room states
pub fn cleanup_lobby(
    mut lobby_state: ResMut<LobbyState>,
) {
    // Reset lobby state for next time
    lobby_state.screen = LobbyScreen::Main;
    lobby_state.error_message = None;
    lobby_state.is_loading = false;
    lobby_state.appearance_initialized = false;
}
