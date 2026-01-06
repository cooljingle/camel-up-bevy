#[cfg(not(target_arch = "wasm32"))]
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::window::WindowMode;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
#[cfg(not(target_arch = "wasm32"))]
use winit::window::Icon;

mod components;
mod game;
mod systems;
mod ui;

use components::{BoardPosition, Camel};
use game::ai::{ai_decision_system, AiConfig, AiThinkTimer};
use game::state::GameState;
use systems::animation::{
    animate_camera_zoom, animate_movement_system, animate_multi_step_movement_system,
    animate_pyramid_hover, animate_pyramid_setup_pulse, animate_pyramid_shake, crown_drop_system,
    dice_result_popup_system, dice_roll_animation_system, explosion_particle_system,
    fade_out_system, firework_system, particle_system, CameraZoomAnimation,
};
use systems::leg::calculate_final_scores;
use systems::movement::{
    move_camel_system, move_crazy_camel_system, MoveCamelEvent, MoveCrazyCamelEvent,
    MovementCompleteEvent,
};
use systems::setup::{
    cleanup_game, hide_setup_instructions_system, initial_roll_animation_system, setup_game,
};
use systems::turn::{
    advance_turn_system, check_game_end_system, check_leg_end_system, game_end_delay_system,
    handle_leg_bet_action, handle_pyramid_click, handle_pyramid_hover, handle_pyramid_roll_action,
    handle_race_bet_action, handle_spectator_tile_action, handle_spectator_tile_clicks,
    update_spectator_tile_sprites, CrazyCamelRollResult, PlaceRaceBetAction,
    PlaceSpectatorTileAction, PlayerLegBetsStore, PlayerPyramidTokens, PyramidRollResult,
    RollPyramidAction, TakeLegBetAction, TurnState,
};
use ui::hud::{
    game_hud_ui, leg_scoring_modal_ui, update_camel_position_animations, update_dice_popup_timer,
    update_ui_on_crazy_roll, update_ui_on_roll, CamelPositionAnimations, UiState,
};
use ui::main_menu::main_menu_ui;
use ui::player_setup::PlayerSetupConfig;
use ui::rules::RulesState;
use ui::scoring::{game_end_ui, setup_game_end_state, CelebrationState};
use ui::theme::{configure_fonts, FontsConfigured};

fn main() {
    let mut app = App::new();

    // Configure window based on platform
    #[cfg(target_arch = "wasm32")]
    let window_config = Window {
        title: "Camel Up".to_string(),
        // Use Windowed mode - canvas size controlled by CSS/HTML
        mode: WindowMode::Windowed,
        // Critical: fit canvas to parent element (body) which CSS sets to 100vw x 100vh
        fit_canvas_to_parent: true,
        // Prevent default browser behavior (like scrolling on spacebar)
        prevent_default_event_handling: true,
        // Don't set a fixed resolution - let it adapt to canvas size
        ..default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    let window_config = Window {
        title: "Camel Up".to_string(),
        resolution: bevy::window::WindowResolution::new(1280, 720),
        ..default()
    };

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(window_config),
        ..default()
    }))
    .add_plugins(EguiPlugin::default())
    // Game states
    .init_state::<GameState>()
    // Resources
    .init_resource::<UiState>()
    .init_resource::<CamelPositionAnimations>()
    .init_resource::<PlayerSetupConfig>()
    .init_resource::<AiConfig>()
    .init_resource::<AiThinkTimer>()
    .init_resource::<CelebrationState>()
    .init_resource::<RulesState>()
    .init_resource::<FontsConfigured>()
    .init_resource::<CameraState>()
    // Messages
    .add_message::<MoveCamelEvent>()
    .add_message::<MoveCrazyCamelEvent>()
    .add_message::<MovementCompleteEvent>()
    .add_message::<TakeLegBetAction>()
    .add_message::<PlaceSpectatorTileAction>()
    .add_message::<RollPyramidAction>()
    .add_message::<PlaceRaceBetAction>()
    .add_message::<PyramidRollResult>()
    .add_message::<CrazyCamelRollResult>();

    // Startup systems (platform-specific)
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Startup, (setup_camera, set_window_icon));
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Startup, setup_camera);

    // UI and camera scaling systems - runs every frame to handle window resizing
    // Font configuration also runs in Update but only configures once
    app.add_systems(
        Update,
        (scale_ui_to_fit, scale_camera_to_fit, configure_fonts),
    );

    // Game setup when entering Playing state
    app.add_systems(OnEnter(GameState::Playing), setup_game_with_resources)
        // UI systems (egui context pass)
        .add_systems(
            EguiPrimaryContextPass,
            main_menu_ui.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            game_hud_ui.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            leg_scoring_modal_ui.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            game_end_ui.run_if(in_state(GameState::GameEnd)),
        )
        // Game logic systems (Update schedule)
        .add_systems(
            Update,
            (
                handle_leg_bet_action,
                handle_pyramid_roll_action,
                handle_pyramid_click,
                handle_pyramid_hover,
                handle_race_bet_action,
                handle_spectator_tile_action,
                update_spectator_tile_sprites,
                handle_spectator_tile_clicks,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            move_camel_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            move_crazy_camel_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            update_ui_on_roll.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            update_ui_on_crazy_roll.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            update_dice_popup_timer.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            update_camel_position_animations.run_if(in_state(GameState::Playing)),
        )
        // AI decision system - runs when it's an AI player's turn
        .add_systems(
            Update,
            ai_decision_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            advance_turn_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            check_leg_end_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            check_game_end_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_end_delay_system.run_if(in_state(GameState::Playing)),
        )
        // Initial roll animation system
        .add_systems(
            Update,
            initial_roll_animation_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            hide_setup_instructions_system.run_if(in_state(GameState::Playing)),
        )
        // Animation systems (run in all states for smooth animations)
        .add_systems(Update, animate_movement_system)
        .add_systems(Update, animate_multi_step_movement_system)
        .add_systems(Update, animate_pyramid_shake)
        .add_systems(Update, animate_pyramid_hover)
        .add_systems(Update, animate_pyramid_setup_pulse)
        .add_systems(Update, animate_camera_zoom)
        .add_systems(Update, fade_out_system)
        .add_systems(Update, dice_result_popup_system)
        .add_systems(Update, dice_roll_animation_system)
        .add_systems(Update, particle_system)
        .add_systems(Update, firework_system)
        .add_systems(Update, explosion_particle_system)
        .add_systems(Update, crown_drop_system)
        // Game end scoring
        .add_systems(
            OnEnter(GameState::GameEnd),
            (calculate_final_scores, setup_game_end_state),
        )
        // Cleanup when returning to main menu
        .add_systems(OnEnter(GameState::MainMenu), cleanup_game)
        .run();
}

// Design resolution - the game is designed for these sizes
const DESIGN_WIDTH: f32 = 1280.0;
const DESIGN_HEIGHT: f32 = 720.0;
const MOBILE_DESIGN_WIDTH: f32 = 400.0;
const MOBILE_DESIGN_HEIGHT: f32 = 600.0;

// Layout thresholds
const MIN_SIDE_PANEL_WIDTH: f32 = 600.0; // Minimum width to use side panels
const SIDE_PANEL_ASPECT_RATIO: f32 = 1.2; // Minimum aspect ratio for side panels

/// Resource to track camera state for zoom transitions
#[derive(Resource, Default)]
pub struct CameraState {
    /// Tracks the previous value of initial_rolls_complete to detect transitions
    last_initial_rolls_complete: bool,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// System to scale the entire UI (egui) based on window size
/// Uses aspect ratio to determine layout (side panels vs top/bottom)
/// Scales UI based on both width and height constraints
fn scale_ui_to_fit(
    mut egui_contexts: Query<&mut bevy_egui::EguiContextSettings>,
    windows: Query<&Window>,
    mut ui_state: ResMut<ui::hud::UiState>,
) {
    let Ok(window) = windows.single() else { return };

    let window_width = window.width();
    let window_height = window.height();

    if window_width <= 0.0 || window_height <= 0.0 {
        return;
    }

    // Determine layout based on aspect ratio and minimum width
    let aspect_ratio = window_width / window_height;
    let use_side_panels =
        aspect_ratio > SIDE_PANEL_ASPECT_RATIO && window_width >= MIN_SIDE_PANEL_WIDTH;
    ui_state.use_side_panels = use_side_panels;

    // Calculate UI scale with height constraint
    let scale = if use_side_panels {
        // Landscape: scale based on smaller dimension ratio
        let scale_x = window_width / DESIGN_WIDTH;
        let scale_y = window_height / DESIGN_HEIGHT;
        (scale_x.min(scale_y) * 0.95).clamp(0.5, 1.5)
    } else {
        // Portrait: scale based on width, but cap by height
        let width_scale = window_width / MOBILE_DESIGN_WIDTH;
        let height_scale = window_height / MOBILE_DESIGN_HEIGHT;
        width_scale.min(height_scale).max(1.0)
    };

    // Set egui's scale factor for all contexts
    for mut settings in egui_contexts.iter_mut() {
        settings.scale_factor = scale;
    }
}

// Game board design dimensions
const GAME_WORLD_HEIGHT: f32 = 400.0; // Board height + stack space + margins

// Board layout constants
const BOARD_START_X: f32 = -280.0; // X position of space 0
const BOARD_SPACING: f32 = 80.0; // Distance between spaces
const BOARD_MARGIN: f32 = 102.0; // Margin on each side (reduced for mobile breathing room)

/// Calculate the X range of the board that should be visible based on game state
/// Returns (min_x, max_x) in world coordinates
fn calculate_visible_board_range(
    ui_state: &UiState,
    camels: &Query<&BoardPosition, With<Camel>>,
    current_game_state: &GameState,
) -> (f32, f32) {
    // Default: show spaces 1-16 (main track, not space 0)
    let default_min = BOARD_START_X + BOARD_SPACING; // -200 (space 1)
    let default_max = BOARD_START_X + 7.0 * BOARD_SPACING; // +280 (space 7/8)

    // Check if any camel is at space 0 (initial setup)
    let has_camel_at_start = camels.iter().any(|p| p.space_index == 0);

    // Check if game has ended (a camel crossed the finish line)
    // Note: We use GameState::GameEnd instead of checking space_index >= 16
    // because space_index is clamped to 15 in the movement system
    let has_camel_past_finish = matches!(current_game_state, GameState::GameEnd);

    // Also check if we're still in initial setup
    let in_initial_setup = !ui_state.initial_rolls_complete;

    // Determine min_x based on state
    let min_x = if has_camel_past_finish {
        -400.0 // Include winner position at -360 with margin
    } else if has_camel_at_start || in_initial_setup {
        -500.0 // Include staging position at -450 with margin
    } else {
        default_min // -200, skip space 0
    };

    (min_x, default_max)
}

/// System to scale the camera's orthographic projection to fit the game board
/// Uses the measured game_board_rect from egui CentralPanel for accurate sizing
/// Dynamically adjusts visible range based on game state
/// Triggers smooth zoom animation when initial rolls complete

fn scale_camera_to_fit(
    mut camera_query: Query<
        (Entity, &mut Projection, Option<&CameraZoomAnimation>),
        With<Camera2d>,
    >,
    ui_state: Res<ui::hud::UiState>,
    camels: Query<&BoardPosition, With<Camel>>,
    mut camera_state: ResMut<CameraState>,
    mut commands: Commands,
    current_game_state: Res<State<GameState>>,
) {
    let Ok((entity, mut projection, animation)) = camera_query.single_mut() else {
        return;
    };

    let Some(rect) = ui_state.game_board_rect else {
        return;
    };

    let effective_width = rect.width();
    let effective_height = rect.height();

    if effective_width <= 0.0 || effective_height <= 0.0 {
        return;
    }

    // 1. Calculate the authoritative target based on state
    let (world_min_x, world_max_x) = if ui_state.initial_rolls_complete {
        (
            BOARD_START_X + BOARD_SPACING,
            BOARD_START_X + 7.0 * BOARD_SPACING,
        )
    } else {
        calculate_visible_board_range(ui_state.as_ref(), &camels, current_game_state.get())
    };

    let world_width = (world_max_x - world_min_x) + 2.0 * BOARD_MARGIN;
    let scale_x = world_width / effective_width;
    let scale_y = GAME_WORLD_HEIGHT / effective_height;
    let target_scale = scale_x.max(scale_y).max(1.0);

    // 2. Handle Transition
    if ui_state.initial_rolls_complete && !camera_state.last_initial_rolls_complete {
        if let Projection::Orthographic(ref ortho) = *projection {
            let current_scale = ortho.scale;

            if (current_scale - target_scale).abs() > 0.01 {
                bevy::log::info!(
                    "Starting zoom animation: {:.3} -> {:.3}",
                    current_scale,
                    target_scale
                );
                commands.entity(entity).insert(CameraZoomAnimation::new(
                    current_scale,
                    target_scale,
                    0.2,
                ));
            }
        }
        camera_state.last_initial_rolls_complete = true;

        // --- CRITICAL FIX ---
        // Return immediately! Do not fall through to the steady-state logic below.
        // The 'CameraZoomAnimation' component won't exist on the entity until
        // the next frame (Commands are deferred), so 'animation.is_some()'
        // below would be false, causing an instant snap if we didn't return here.
        return;
    }

    // Reset tracking when returning to initial setup
    if !ui_state.initial_rolls_complete && camera_state.last_initial_rolls_complete {
        camera_state.last_initial_rolls_complete = false;
    }

    // 3. Steady State Application
    // If animation is running, do nothing (animation system handles it)
    if animation.is_some() {
        return;
    }

    // Otherwise, apply target immediately
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = target_scale;
    }
}

// ============================================================================
// Window Icon (Native only - not available on WASM)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
/// Set the window icon from PNG file
fn set_window_icon(_: NonSendMarker) {
    // Try to load icon from file
    let icon = load_icon_from_file("assets/icon.png").or_else(|| {
        // Fallback: create a simple camel-colored icon programmatically
        create_camel_icon()
    });

    if let Some(icon) = icon {
        bevy::winit::WINIT_WINDOWS.with_borrow_mut(|winit_windows| {
            for window in winit_windows.windows.values() {
                window.set_window_icon(Some(icon.clone()));
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Load icon from PNG file
fn load_icon_from_file(path: &str) -> Option<Icon> {
    let image = image::open(path).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).ok()
}

#[cfg(not(target_arch = "wasm32"))]
/// Create a pyramid icon programmatically (32x32)
fn create_camel_icon() -> Option<Icon> {
    const SIZE: u32 = 32;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    // Colors
    let sky_blue = [0x87, 0xCE, 0xEB, 0xFF]; // Sky background
    let sand = [0xED, 0xC9, 0x9A, 0xFF]; // Desert sand
    let pyramid_light = [0xD4, 0xA8, 0x4B, 0xFF]; // Pyramid sunlit side
    let pyramid_dark = [0xA0, 0x7A, 0x30, 0xFF]; // Pyramid shadow side
    let pyramid_outline = [0x6B, 0x4A, 0x1A, 0xFF]; // Pyramid edge
    let sun = [0xFF, 0xD7, 0x00, 0xFF]; // Sun

    // Pyramid geometry (centered, apex at top)
    let apex_x = 16.0f32;
    let apex_y = 4.0f32;
    let base_left = 2.0f32;
    let base_right = 30.0f32;
    let base_y = 26.0f32;
    let horizon_y = 26;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;
            let fx = x as f32;
            let fy = y as f32;

            // Check if point is inside pyramid using barycentric coordinates
            let in_pyramid = point_in_triangle(
                fx, fy, apex_x, apex_y, base_left, base_y, base_right, base_y,
            );

            // Determine which side of pyramid (left = shadow, right = lit)
            let on_left_side = fx < apex_x;

            // Distance to pyramid edges for outline detection
            let dist_to_left_edge = distance_to_line(fx, fy, apex_x, apex_y, base_left, base_y);
            let dist_to_right_edge = distance_to_line(fx, fy, apex_x, apex_y, base_right, base_y);
            let dist_to_base = if fy >= base_y - 1.0 && fy <= base_y + 1.0 {
                0.0
            } else {
                2.0
            };
            let near_edge =
                dist_to_left_edge < 1.2 || dist_to_right_edge < 1.2 || dist_to_base < 1.0;

            // Sun in upper right
            let sun_cx = 26.0f32;
            let sun_cy = 6.0f32;
            let sun_dist = ((fx - sun_cx).powi(2) + (fy - sun_cy).powi(2)).sqrt();

            let color = if sun_dist < 3.5 {
                sun
            } else if in_pyramid {
                if near_edge {
                    pyramid_outline
                } else if on_left_side {
                    pyramid_dark
                } else {
                    pyramid_light
                }
            } else if y >= horizon_y {
                sand
            } else {
                sky_blue
            };

            rgba[idx..idx + 4].copy_from_slice(&color);
        }
    }

    Icon::from_rgba(rgba, SIZE, SIZE).ok()
}

#[cfg(not(target_arch = "wasm32"))]
/// Check if point (px, py) is inside triangle with vertices (x1,y1), (x2,y2), (x3,y3)
fn point_in_triangle(
    px: f32,
    py: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
) -> bool {
    let area = 0.5 * (-y2 * x3 + y1 * (-x2 + x3) + x1 * (y2 - y3) + x2 * y3);
    let s = 1.0 / (2.0 * area) * (y1 * x3 - x1 * y3 + (y3 - y1) * px + (x1 - x3) * py);
    let t = 1.0 / (2.0 * area) * (x1 * y2 - y1 * x2 + (y1 - y2) * px + (x2 - x1) * py);
    s >= 0.0 && t >= 0.0 && (s + t) <= 1.0
}

#[cfg(not(target_arch = "wasm32"))]
/// Distance from point (px, py) to line segment from (x1, y1) to (x2, y2)
fn distance_to_line(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0.0 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    let t = ((px - x1) * dx + (py - y1) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

/// Setup game with additional resources for turn tracking
fn setup_game_with_resources(
    commands: Commands,
    config: Res<PlayerSetupConfig>,
    existing_camels: Query<Entity, With<components::Camel>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    setup_game(commands, config, existing_camels, meshes, materials, asset_server);
}

/// Initialize turn-related resources
pub fn init_turn_resources(mut commands: Commands, players: Res<components::Players>) {
    let player_count = players.players.len();
    commands.insert_resource(TurnState::default());
    commands.insert_resource(PlayerLegBetsStore::new(player_count));
    commands.insert_resource(PlayerPyramidTokens::new(player_count));
}
