//! UI state types and animation data structures for the game HUD.

use bevy::prelude::*;
use bevy_egui::egui;
use crate::components::CamelColor;

/// Represents the last die roll result (regular or crazy camel)
#[derive(Clone)]
pub enum LastRoll {
    Regular(CamelColor, u8),
    Crazy(crate::components::CrazyCamelColor, u8),
}

/// Mobile navigation tabs (deprecated - keeping for backwards compatibility)
#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MobileTab {
    #[default]
    Actions,
    Players,
    Positions,
}

/// Animation state for die being rolled from pyramid
#[derive(Clone, Copy)]
pub struct DieRollAnimation {
    pub die_color: Option<CamelColor>,  // None = crazy die (gray)
    pub start_time: f64,                // When animation started (seconds)
}

/// Phase of card flight animation
#[derive(Clone, Copy, PartialEq)]
pub enum CardFlightPhase {
    FlyingToPanel,     // 0.0 - 0.25s: Card flies from stack to panel edge
    DisappearingUnder, // 0.25 - 0.32s: Card shrinks/fades as it goes "under"
    ReappearingInside, // 0.32 - 0.40s: Mini card fades in inside player's collection
    Done,
}

/// Animation state for leg bet card flying to player
#[derive(Clone, Copy)]
pub struct CardFlightAnimation {
    pub color: CamelColor,
    pub value: u8,
    pub start_pos: egui::Pos2,      // Card stack position (screen coords)
    pub end_pos: egui::Pos2,        // Player panel edge
    pub start_time: f64,
    pub phase: CardFlightPhase,
}

/// UI state for showing different panels
#[derive(Resource)]
pub struct UiState {
    pub show_winner_betting: bool,   // Show winner bet modal
    pub show_loser_betting: bool,    // Show loser bet modal
    pub show_spectator_tile: bool,
    pub spectator_tile_space: Option<u8>,  // Selected space for spectator tile
    pub spectator_tile_is_oasis: bool,     // Current side of spectator tile card (true = oasis +1)
    pub spectator_tile_flip_anim: f32,     // Animation progress for card flip (0.0 to 1.0)
    pub spectator_tile_selected: bool,     // Whether spectator tile card is selected for placement (mobile)
    pub last_roll: Option<LastRoll>,
    pub dice_popup_delay: f32,          // Delay before showing popup (waits for shake animation)
    pub dice_popup_timer: f32,          // Timer for dice result popup fade
    pub show_leg_scoring: bool,         // Show leg scoring modal
    pub show_rules: bool,               // Show game rules modal
    pub initial_rolls_complete: bool,   // Whether initial setup rolls have finished
    pub exit_fullscreen_requested: bool, // Request to exit fullscreen mode
    pub enter_fullscreen_requested: bool, // Request to enter fullscreen mode
    pub use_side_panels: bool,          // Layout mode: true = side panels (landscape), false = top/bottom (portrait)
    pub game_board_rect: Option<egui::Rect>, // Measured game board area from CentralPanel
    #[allow(dead_code)]
    pub mobile_tab: MobileTab,          // Current tab in mobile view (deprecated)
    pub die_roll_animation: Option<DieRollAnimation>,  // Animation for die being selected/rolled
    pub pyramid_flip_anim: f32,  // 0.0 = not animating, 0.01-1.0 = flip in progress
    pub card_flight_animation: Option<CardFlightAnimation>,  // Animation for leg bet card flying to player
    pub leg_bet_card_positions: [Option<egui::Pos2>; 5],  // Screen positions of leg bet card stacks (indexed by CamelColor)
    pub player_bet_area_pos: Option<egui::Pos2>,  // Screen position where player's bets are displayed
    pub show_debug_overlay: bool,  // Show debug overlay with window dimensions
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_winner_betting: false,
            show_loser_betting: false,
            show_spectator_tile: false,
            spectator_tile_space: None,
            spectator_tile_is_oasis: true,  // Start with oasis side (+1)
            spectator_tile_flip_anim: 0.0,
            spectator_tile_selected: false,
            last_roll: None,
            dice_popup_delay: 0.0,
            dice_popup_timer: 0.0,
            show_leg_scoring: false,
            show_rules: false,
            initial_rolls_complete: false,
            exit_fullscreen_requested: false,
            enter_fullscreen_requested: false,
            use_side_panels: true,  // Default to side panels (landscape)
            game_board_rect: None,
            mobile_tab: MobileTab::default(),
            die_roll_animation: None,
            pyramid_flip_anim: 0.0,
            card_flight_animation: None,
            leg_bet_card_positions: [None; 5],
            player_bet_area_pos: None,
            show_debug_overlay: false,
        }
    }
}

/// Animated position entry for the camel positions panel
#[derive(Clone, Copy)]
pub struct AnimatedCamelPosition {
    pub color: CamelColor,
    pub current_y_offset: f32,  // Current Y position offset for animation
    pub target_y_offset: f32,   // Target Y position (0 = at rank position)
}

/// Resource for tracking camel position animations in UI
#[derive(Resource, Default)]
pub struct CamelPositionAnimations {
    pub positions: Vec<AnimatedCamelPosition>,
    pub last_order: Vec<CamelColor>,  // Previous frame's order for detecting changes
}
