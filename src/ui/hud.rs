use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::components::{Players, Pyramid, LegBettingTiles, RaceBets, CamelColor, CrazyCamelColor, Camel, BoardPosition, PlacedDesertTiles, DesertTile, TRACK_LENGTH};
use crate::components::dice::PyramidDie;
use crate::systems::turn::{TurnState, RollPyramidAction, TakeLegBetAction, PlaceRaceBetAction, PlaceDesertTileAction, PyramidRollResult, CrazyCamelRollResult, PlayerLegBetsStore, PlayerPyramidTokens};
use crate::systems::movement::{get_leading_camel, get_second_place_camel};
use crate::game::state::GameState;
use crate::ui::characters::{draw_avatar, CharacterId};

/// Player colors for visual distinction
const PLAYER_COLORS: [egui::Color32; 8] = [
    egui::Color32::from_rgb(220, 50, 50),   // Red
    egui::Color32::from_rgb(50, 120, 220),  // Blue
    egui::Color32::from_rgb(50, 180, 80),   // Green
    egui::Color32::from_rgb(220, 180, 50),  // Yellow
    egui::Color32::from_rgb(180, 80, 220),  // Purple
    egui::Color32::from_rgb(220, 130, 50),  // Orange
    egui::Color32::from_rgb(80, 200, 200),  // Cyan
    egui::Color32::from_rgb(200, 100, 150), // Pink
];

/// Convert CamelColor to egui Color32 for UI display
pub fn camel_color_to_egui(color: CamelColor) -> egui::Color32 {
    match color {
        CamelColor::Blue => egui::Color32::from_rgb(50, 100, 230),
        CamelColor::Green => egui::Color32::from_rgb(50, 200, 80),
        CamelColor::Red => egui::Color32::from_rgb(230, 50, 50),
        CamelColor::Yellow => egui::Color32::from_rgb(240, 230, 50),
        CamelColor::Purple => egui::Color32::from_rgb(150, 50, 200),
    }
}

/// Convert CrazyCamelColor to egui Color32 for UI display
fn crazy_camel_color_to_egui(color: CrazyCamelColor) -> egui::Color32 {
    match color {
        CrazyCamelColor::Black => egui::Color32::from_rgb(40, 40, 40),
        CrazyCamelColor::White => egui::Color32::from_rgb(240, 240, 240),
    }
}

/// Helper function to draw a small camel silhouette for UI elements
/// Draws a stylized side-view camel using simple shapes
pub fn draw_camel_silhouette(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32, border_color: egui::Color32) {
    let center = rect.center();
    let scale = (rect.width().min(rect.height()) / 30.0).min(1.0);

    // Body - main oval
    let body_width = 16.0 * scale;
    let body_height = 9.0 * scale;
    let body_center = center + egui::vec2(-2.0 * scale, 2.0 * scale);
    let body_rect = egui::Rect::from_center_size(body_center, egui::vec2(body_width, body_height));

    // Hump - on top of body
    let hump_width = 8.0 * scale;
    let hump_height = 7.0 * scale;
    let hump_center = body_center + egui::vec2(-1.0 * scale, -6.0 * scale);
    let hump_rect = egui::Rect::from_center_size(hump_center, egui::vec2(hump_width, hump_height));

    // Neck - tall narrow rectangle
    let neck_width = 4.0 * scale;
    let neck_height = 9.0 * scale;
    let neck_center = body_center + egui::vec2(8.0 * scale, -4.0 * scale);
    let neck_rect = egui::Rect::from_center_size(neck_center, egui::vec2(neck_width, neck_height));

    // Head - small oval
    let head_width = 7.0 * scale;
    let head_height = 5.0 * scale;
    let head_center = neck_center + egui::vec2(3.0 * scale, -5.0 * scale);
    let head_rect = egui::Rect::from_center_size(head_center, egui::vec2(head_width, head_height));

    // Legs - four thin rectangles
    let leg_width = 2.5 * scale;
    let leg_height = 7.0 * scale;
    let leg_positions = [
        body_center + egui::vec2(-5.0 * scale, 7.0 * scale),   // Back left
        body_center + egui::vec2(-2.0 * scale, 7.0 * scale),   // Back right
        body_center + egui::vec2(4.0 * scale, 7.0 * scale),    // Front left
        body_center + egui::vec2(7.0 * scale, 7.0 * scale),    // Front right
    ];

    // Draw border layer (slightly larger)
    let border_expand = 1.5 * scale;
    painter.rect_filled(body_rect.expand(border_expand), 4.0 * scale, border_color);
    painter.rect_filled(hump_rect.expand(border_expand), 3.0 * scale, border_color);
    painter.rect_filled(neck_rect.expand(border_expand), 2.0 * scale, border_color);
    painter.rect_filled(head_rect.expand(border_expand), 3.0 * scale, border_color);
    for leg_pos in &leg_positions {
        let leg_rect = egui::Rect::from_center_size(*leg_pos, egui::vec2(leg_width, leg_height));
        painter.rect_filled(leg_rect.expand(border_expand * 0.5), 1.0 * scale, border_color);
    }

    // Draw main color layer
    painter.rect_filled(body_rect, 4.0 * scale, color);
    painter.rect_filled(hump_rect, 3.0 * scale, color);
    painter.rect_filled(neck_rect, 2.0 * scale, color);
    painter.rect_filled(head_rect, 3.0 * scale, color);
    for leg_pos in &leg_positions {
        let leg_rect = egui::Rect::from_center_size(*leg_pos, egui::vec2(leg_width, leg_height));
        painter.rect_filled(leg_rect, 1.0 * scale, color);
    }

    // Eye
    let eye_pos = head_center + egui::vec2(1.5 * scale, -0.5 * scale);
    painter.circle_filled(eye_pos, 1.0 * scale, egui::Color32::from_rgb(30, 30, 30));
}

/// Helper function to draw a mini leg bet card (camel silhouette on top, value on bottom)
pub fn draw_mini_leg_bet_card(painter: &egui::Painter, rect: egui::Rect, camel_color: CamelColor, value: u8) {
    let color = camel_color_to_egui(camel_color);
    let border_color = egui::Color32::from_rgb(
        (color.r() as f32 * 0.5) as u8,
        (color.g() as f32 * 0.5) as u8,
        (color.b() as f32 * 0.5) as u8,
    );

    // Card border/shadow
    painter.rect_filled(rect.expand(1.0), 3.0, egui::Color32::from_rgb(60, 50, 40));
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(245, 235, 215));

    // Split into top (camel) and bottom (value)
    let top_half = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.max.x, rect.center().y + 2.0)
    );
    let bottom_half = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, rect.center().y + 2.0),
        rect.max
    );

    // Top half - cream for camel
    painter.rect_filled(top_half.shrink(1.0), 1.0, egui::Color32::from_rgb(250, 245, 230));

    // Draw camel silhouette
    let camel_rect = egui::Rect::from_min_size(
        top_half.min + egui::vec2(2.0, 2.0),
        egui::vec2(top_half.width() - 4.0, top_half.height() - 4.0)
    );
    draw_camel_silhouette(painter, camel_rect, color, border_color);

    // Bottom half - colored band with value
    painter.rect_filled(bottom_half.shrink2(egui::vec2(1.0, 0.0)), 1.0, color);

    // Value text
    let text_color = if camel_color == CamelColor::Yellow {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    };
    painter.text(
        bottom_half.center(),
        egui::Align2::CENTER_CENTER,
        format!("{}", value),
        egui::FontId::proportional(10.0),
        text_color,
    );
}

/// Helper function to draw a small pyramid token icon
/// Returns the rect used for the icon
pub fn draw_pyramid_token_icon(painter: &egui::Painter, center: egui::Pos2, size: f32) {
    let pyramid_height = size * 0.8;
    let pyramid_width = size * 0.7;

    let apex = egui::pos2(center.x, center.y - pyramid_height / 2.0);
    let base_left = egui::pos2(center.x - pyramid_width / 2.0, center.y + pyramid_height / 2.0);
    let base_right = egui::pos2(center.x + pyramid_width / 2.0, center.y + pyramid_height / 2.0);
    let mid_base = egui::pos2(center.x, center.y + pyramid_height / 2.0);

    // Gold/sand colored pyramid token
    let pyramid_color = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);
    let shadow_color = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);
    let outline_color = egui::Color32::from_rgb(0x6B, 0x4A, 0x1A);

    // Draw shadow/depth on left side
    painter.add(egui::Shape::convex_polygon(
        vec![apex, base_left, mid_base],
        shadow_color,
        egui::Stroke::NONE,
    ));

    // Draw lit side on right
    painter.add(egui::Shape::convex_polygon(
        vec![apex, mid_base, base_right],
        pyramid_color,
        egui::Stroke::NONE,
    ));

    // Draw outline
    painter.add(egui::Shape::closed_line(
        vec![apex, base_left, base_right],
        egui::Stroke::new(1.0, outline_color),
    ));

    // Draw "$1" on the token
    painter.text(
        egui::pos2(center.x, center.y + 2.0),
        egui::Align2::CENTER_CENTER,
        "$1",
        egui::FontId::proportional(size * 0.28),
        outline_color,
    );
}

/// Helper function to draw a desert tile card with player avatar on top and +1/-1 on bottom
/// flip_progress: 0.0 = front fully visible, 1.0 = back fully visible
/// When flip_progress is 0.0-0.5, we show front side scaling down; 0.5-1.0 shows back side scaling up
pub fn draw_desert_tile_card(
    painter: &egui::Painter,
    rect: egui::Rect,
    character_id: CharacterId,
    player_color: egui::Color32,
    is_oasis: bool,
    flip_progress: f32,
) {
    // Calculate horizontal scale for flip effect (simulates 3D rotation)
    let (show_front, scale_x) = if flip_progress <= 0.5 {
        // First half: showing front, shrinking
        let t = flip_progress * 2.0; // 0.0 to 1.0
        (true, 1.0 - t)
    } else {
        // Second half: showing back, growing
        let t = (flip_progress - 0.5) * 2.0; // 0.0 to 1.0
        (false, t)
    };

    // If scale is too small, don't draw (card is edge-on)
    if scale_x < 0.05 {
        return;
    }

    // Calculate scaled rect (squeeze horizontally from center)
    let center_x = rect.center().x;
    let new_width = rect.width() * scale_x;
    let scaled_rect = egui::Rect::from_center_size(
        egui::pos2(center_x, rect.center().y),
        egui::vec2(new_width, rect.height()),
    );

    // Determine which side to show based on is_oasis and show_front
    let showing_oasis = if show_front { is_oasis } else { !is_oasis };

    // Colors for each side
    let (bg_color, value_color, value_text) = if showing_oasis {
        // Oasis (+1): Green/lush colors
        (
            egui::Color32::from_rgb(80, 160, 80),   // Green
            egui::Color32::from_rgb(50, 120, 50),   // Dark green
            "+1",
        )
    } else {
        // Mirage (-1): Sandy/orange colors
        (
            egui::Color32::from_rgb(200, 150, 80),  // Sandy
            egui::Color32::from_rgb(160, 100, 40),  // Dark sandy
            "-1",
        )
    };

    // Card shadow
    painter.rect_filled(scaled_rect.expand(2.0), 5.0, egui::Color32::from_rgb(40, 30, 20));

    // Card background
    painter.rect_filled(scaled_rect, 4.0, egui::Color32::from_rgb(245, 235, 215));

    // Split into top (avatar) and bottom (value)
    let top_half = egui::Rect::from_min_max(
        scaled_rect.min,
        egui::pos2(scaled_rect.max.x, scaled_rect.center().y + 4.0),
    );
    let bottom_half = egui::Rect::from_min_max(
        egui::pos2(scaled_rect.min.x, scaled_rect.center().y + 4.0),
        scaled_rect.max,
    );

    // Top half - cream background with avatar
    painter.rect_filled(top_half.shrink(2.0), 2.0, egui::Color32::from_rgb(250, 245, 230));

    // Draw player avatar in top portion (only if wide enough)
    if scale_x > 0.3 {
        let avatar_rect = egui::Rect::from_min_size(
            top_half.min + egui::vec2(4.0 * scale_x, 4.0),
            egui::vec2((top_half.width() - 8.0 * scale_x).max(8.0), top_half.height() - 8.0),
        );
        draw_avatar(painter, avatar_rect, character_id, Some(player_color));
    }

    // Bottom half - colored band with value
    painter.rect_filled(bottom_half.shrink2(egui::vec2(2.0, 0.0)), 2.0, bg_color);

    // Draw +1 or -1 text
    painter.text(
        bottom_half.center(),
        egui::Align2::CENTER_CENTER,
        value_text,
        egui::FontId::proportional(14.0 * scale_x.max(0.5)),
        egui::Color32::WHITE,
    );

    // Draw small icon indicator (palm tree for oasis, wavy lines for mirage)
    if scale_x > 0.4 {
        let icon_center = bottom_half.center() + egui::vec2(new_width * 0.25, 0.0);
        if showing_oasis {
            // Palm tree icon (simple)
            let trunk_bottom = icon_center + egui::vec2(0.0, 6.0);
            let trunk_top = icon_center + egui::vec2(0.0, -2.0);
            painter.line_segment([trunk_bottom, trunk_top], egui::Stroke::new(2.0 * scale_x, value_color));
            // Fronds
            painter.line_segment([trunk_top, trunk_top + egui::vec2(-5.0 * scale_x, -4.0)], egui::Stroke::new(1.5 * scale_x, value_color));
            painter.line_segment([trunk_top, trunk_top + egui::vec2(5.0 * scale_x, -4.0)], egui::Stroke::new(1.5 * scale_x, value_color));
            painter.line_segment([trunk_top, trunk_top + egui::vec2(0.0, -5.0)], egui::Stroke::new(1.5 * scale_x, value_color));
        } else {
            // Wavy mirage lines
            for i in 0..3 {
                let y_offset = (i as f32 - 1.0) * 4.0;
                let start = icon_center + egui::vec2(-4.0 * scale_x, y_offset);
                let end = icon_center + egui::vec2(4.0 * scale_x, y_offset);
                painter.line_segment([start, end], egui::Stroke::new(1.5 * scale_x, value_color));
            }
        }
    }
}

/// Helper function to draw dice pips at a given center point
fn draw_dice_pips(ui: &mut egui::Ui, center: egui::Pos2, value: u8, pip_color: egui::Color32, pip_size: f32, pip_spacing: f32) {
    match value {
        1 => {
            ui.painter().circle_filled(center, pip_size, pip_color);
        }
        2 => {
            ui.painter().circle_filled(center + egui::vec2(-pip_spacing, -pip_spacing), pip_size, pip_color);
            ui.painter().circle_filled(center + egui::vec2(pip_spacing, pip_spacing), pip_size, pip_color);
        }
        3 => {
            ui.painter().circle_filled(center + egui::vec2(-pip_spacing, -pip_spacing), pip_size, pip_color);
            ui.painter().circle_filled(center, pip_size, pip_color);
            ui.painter().circle_filled(center + egui::vec2(pip_spacing, pip_spacing), pip_size, pip_color);
        }
        _ => {
            ui.painter().circle_filled(center, pip_size, pip_color);
        }
    }
}

/// Helper function to draw a race bet card (player avatar on camel color background)
fn draw_race_bet_card(
    painter: &egui::Painter,
    rect: egui::Rect,
    camel_color: CamelColor,
    character_id: CharacterId,
    player_color: egui::Color32,
    hovered: bool,
) {
    let color = camel_color_to_egui(camel_color);
    let border_color = egui::Color32::from_rgb(
        (color.r() as f32 * 0.5) as u8,
        (color.g() as f32 * 0.5) as u8,
        (color.b() as f32 * 0.5) as u8,
    );

    // Card shadow
    let shadow_rect = rect.translate(egui::vec2(2.0, 2.0));
    painter.rect_filled(shadow_rect, 6.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60));

    // Card border
    painter.rect_filled(rect.expand(2.0), 6.0, border_color);

    // Card background with camel color
    painter.rect_filled(rect, 5.0, color);

    // Avatar in the center-top area
    let avatar_size = rect.width() * 0.65;
    let avatar_rect = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.center().y - rect.height() * 0.08),
        egui::vec2(avatar_size, avatar_size),
    );
    draw_avatar(painter, avatar_rect, character_id, Some(player_color));

    // Camel name at the bottom
    let text_color = if camel_color == CamelColor::Yellow {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    };
    painter.text(
        egui::pos2(rect.center().x, rect.max.y - 10.0),
        egui::Align2::CENTER_CENTER,
        format!("{:?}", camel_color),
        egui::FontId::proportional(10.0),
        text_color,
    );

    // Hover glow effect
    if hovered {
        painter.rect_stroke(rect.expand(3.0), 6.0, egui::Stroke::new(3.0, egui::Color32::GOLD), egui::epaint::StrokeKind::Outside);
    }
}

/// Helper function to draw an unavailable/used race bet card
fn draw_race_bet_card_unavailable(painter: &egui::Painter, rect: egui::Rect, camel_color: CamelColor) {
    let color = camel_color_to_egui(camel_color);
    let faded_color = egui::Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        60,
    );
    let border_color = egui::Color32::from_rgba_unmultiplied(
        (color.r() as f32 * 0.5) as u8,
        (color.g() as f32 * 0.5) as u8,
        (color.b() as f32 * 0.5) as u8,
        80,
    );

    // Faded card border
    painter.rect_filled(rect.expand(2.0), 6.0, border_color);

    // Faded card background
    painter.rect_filled(rect, 5.0, faded_color);

    // Draw an X to indicate unavailable
    let x_padding = rect.width() * 0.2;
    let stroke = egui::Stroke::new(3.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 180));
    painter.line_segment(
        [rect.left_top() + egui::vec2(x_padding, x_padding), rect.right_bottom() - egui::vec2(x_padding, x_padding)],
        stroke,
    );
    painter.line_segment(
        [rect.right_top() + egui::vec2(-x_padding, x_padding), rect.left_bottom() + egui::vec2(x_padding, -x_padding)],
        stroke,
    );

    // Camel name at the bottom (faded)
    let text_color = egui::Color32::from_rgba_unmultiplied(100, 100, 100, 150);
    painter.text(
        egui::pos2(rect.center().x, rect.max.y - 10.0),
        egui::Align2::CENTER_CENTER,
        format!("{:?}", camel_color),
        egui::FontId::proportional(10.0),
        text_color,
    );
}

/// Represents the last die roll result (regular or crazy camel)
#[derive(Clone)]
pub enum LastRoll {
    Regular(CamelColor, u8),
    Crazy(CrazyCamelColor, u8),
}

/// UI state for showing different panels
#[derive(Resource)]
pub struct UiState {
    pub show_race_betting: bool,
    pub show_desert_tile: bool,
    pub desert_tile_space: Option<u8>,  // Selected space for desert tile
    pub desert_tile_is_oasis: bool,     // Current side of desert tile card (true = oasis +1)
    pub desert_tile_flip_anim: f32,     // Animation progress for card flip (0.0 to 1.0)
    pub last_roll: Option<LastRoll>,
    pub dice_popup_timer: f32,          // Timer for dice result popup fade
    pub show_leg_scoring: bool,         // Show leg scoring modal
    pub initial_rolls_complete: bool,   // Whether initial setup rolls have finished
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

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_race_betting: false,
            show_desert_tile: false,
            desert_tile_space: None,
            desert_tile_is_oasis: true,  // Start with oasis side (+1)
            desert_tile_flip_anim: 0.0,
            last_roll: None,
            dice_popup_timer: 0.0,
            show_leg_scoring: false,
            initial_rolls_complete: false,
        }
    }
}

pub fn game_hud_ui(
    mut contexts: EguiContexts,
    players: Option<Res<Players>>,
    pyramid: Option<Res<Pyramid>>,
    leg_tiles: Option<Res<LegBettingTiles>>,
    turn_state: Option<Res<TurnState>>,
    placed_tiles: Option<Res<PlacedDesertTiles>>,
    player_leg_bets: Option<Res<PlayerLegBetsStore>>,
    player_pyramid_tokens: Option<Res<PlayerPyramidTokens>>,
    race_bets: Option<Res<RaceBets>>,
    mut ui_state: ResMut<UiState>,
    camel_animations: Res<CamelPositionAnimations>,
    actions: (
        MessageWriter<RollPyramidAction>,
        MessageWriter<TakeLegBetAction>,
        MessageWriter<PlaceRaceBetAction>,
        MessageWriter<PlaceDesertTileAction>,
    ),
    camels: Query<(&Camel, &BoardPosition)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (mut roll_action, mut leg_bet_action, mut race_bet_action, mut desert_tile_action) = actions;
    let Some(players) = players else { return };
    let Some(pyramid) = pyramid else { return };
    let Some(leg_tiles) = leg_tiles else { return };
    let Some(turn_state) = turn_state else { return };
    let Some(placed_tiles) = placed_tiles else { return };
    let Some(player_leg_bets) = player_leg_bets else { return };
    let Some(player_pyramid_tokens) = player_pyramid_tokens else { return };
    let Some(race_bets) = race_bets else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Shared current player color (used in multiple places)
    let current_player_color = PLAYER_COLORS[players.current_player_index % PLAYER_COLORS.len()];

    // Top bar - Game info
    egui::TopBottomPanel::top("game_info").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Camel Up");
            ui.separator();
            ui.label(format!("Leg {}", turn_state.leg_number));
            ui.separator();
            let rolls_done = pyramid.rolled_dice.len();
            ui.label(format!("Rolls: {}/5", rolls_done));

            if let Some(ref last_roll) = ui_state.last_roll {
                ui.separator();
                match last_roll {
                    LastRoll::Regular(color, value) => {
                        ui.label(format!("Last roll: {:?} moved {} spaces", color, value));
                    }
                    LastRoll::Crazy(color, value) => {
                        ui.label(format!("Last roll: {:?} crazy camel moved {} backwards!", color, value));
                    }
                }
            }

            // Add flexible spacer to push button to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Back to Menu").clicked() {
                    next_state.set(GameState::MainMenu);
                }
            });
        });
    });

    // Bottom panel - Dice Tents (shows dice in order they were rolled)
    egui::TopBottomPanel::bottom("dice_tents").show(ctx, |ui| {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Dice Tents").strong());
            ui.separator();

            let die_size = 40.0;
            let tent_height = 18.0;
            let total_height = die_size + tent_height;
            let total_tents = 5; // 5 tents - leg ends when 5 regular dice are rolled

            // Draw tents - filled with dice in order they were rolled
            for tent_index in 0..total_tents {
                let (full_rect, _) = ui.allocate_exact_size(egui::vec2(die_size, total_height), egui::Sense::hover());

                // Split into tent roof and dice area
                let tent_rect = egui::Rect::from_min_size(full_rect.min, egui::vec2(die_size, tent_height));
                let dice_rect = egui::Rect::from_min_size(
                    full_rect.min + egui::vec2(0.0, tent_height),
                    egui::vec2(die_size, die_size)
                );

                // Determine tent color based on content
                let (has_die, tent_fill_color, is_filled) = if tent_index < pyramid.rolled_dice.len() {
                    let die = &pyramid.rolled_dice[tent_index];
                    match die {
                        PyramidDie::Regular(d) => (true, camel_color_to_egui(d.color), true),
                        PyramidDie::Crazy { rolled } => {
                            if let Some((crazy_color, _)) = rolled {
                                (true, crazy_camel_color_to_egui(*crazy_color), true)
                            } else {
                                (true, egui::Color32::from_rgb(128, 128, 128), true)
                            }
                        }
                    }
                } else {
                    (false, egui::Color32::from_rgb(80, 70, 60), false)
                };

                // Draw tent roof (triangle)
                let tent_peak = egui::pos2(tent_rect.center().x, tent_rect.top());
                let tent_left = egui::pos2(tent_rect.left() - 3.0, tent_rect.bottom());
                let tent_right = egui::pos2(tent_rect.right() + 3.0, tent_rect.bottom());

                let roof_color = if is_filled {
                    // Darken the tent color for the roof
                    egui::Color32::from_rgb(
                        (tent_fill_color.r() as f32 * 0.7) as u8,
                        (tent_fill_color.g() as f32 * 0.7) as u8,
                        (tent_fill_color.b() as f32 * 0.7) as u8,
                    )
                } else {
                    egui::Color32::from_rgba_unmultiplied(80, 70, 60, 120)
                };

                // Fill tent roof
                ui.painter().add(egui::Shape::convex_polygon(
                    vec![tent_peak, tent_left, tent_right],
                    roof_color,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 50, 40)),
                ));

                if has_die && tent_index < pyramid.rolled_dice.len() {
                    // This tent has a die in it
                    let die = &pyramid.rolled_dice[tent_index];

                    match die {
                        PyramidDie::Regular(d) => {
                            let dice_color = camel_color_to_egui(d.color);

                            // Draw dice with shadow for depth
                            let shadow_offset = egui::vec2(2.0, 2.0);
                            let shadow_rect = dice_rect.translate(shadow_offset);
                            ui.painter().rect_filled(shadow_rect, 5.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 50));

                            // Draw dice background
                            ui.painter().rect_filled(dice_rect, 5.0, dice_color);
                            ui.painter().rect_stroke(dice_rect, 5.0, egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 80)), egui::epaint::StrokeKind::Outside);

                            // Draw pips for the value
                            let pip_color = if d.color == CamelColor::Yellow {
                                egui::Color32::BLACK
                            } else {
                                egui::Color32::WHITE
                            };
                            draw_dice_pips(ui, dice_rect.center(), d.value.unwrap_or(1), pip_color, 4.0, 10.0);
                        }
                        PyramidDie::Crazy { rolled } => {
                            if let Some((crazy_color, value)) = rolled {
                                let dice_color = crazy_camel_color_to_egui(*crazy_color);

                                // Draw dice with shadow
                                let shadow_offset = egui::vec2(2.0, 2.0);
                                let shadow_rect = dice_rect.translate(shadow_offset);
                                ui.painter().rect_filled(shadow_rect, 5.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 50));

                                ui.painter().rect_filled(dice_rect, 5.0, dice_color);
                                ui.painter().rect_stroke(dice_rect, 5.0, egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 80)), egui::epaint::StrokeKind::Outside);

                                let pip_color = if *crazy_color == CrazyCamelColor::White {
                                    egui::Color32::BLACK
                                } else {
                                    egui::Color32::WHITE
                                };
                                draw_dice_pips(ui, dice_rect.center(), *value, pip_color, 4.0, 10.0);
                            }
                        }
                    }
                } else {
                    // Empty tent - draw placeholder for dice slot
                    let empty_color = egui::Color32::from_rgba_unmultiplied(100, 90, 80, 50);
                    ui.painter().rect_filled(dice_rect, 5.0, empty_color);
                    ui.painter().rect_stroke(dice_rect, 5.0, egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(120, 110, 100, 80)), egui::epaint::StrokeKind::Outside);

                    // Draw question mark for empty slot
                    ui.painter().text(
                        dice_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "?",
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_rgba_unmultiplied(140, 130, 120, 120),
                    );
                }

                ui.add_space(8.0);
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Pyramid token stack display
            ui.label(egui::RichText::new("Pyramid Tokens").strong());
            ui.add_space(4.0);

            let token_size = 36.0;
            let total_tokens: usize = 5; // 5 tokens available per leg
            let tokens_taken: u8 = player_pyramid_tokens.counts.iter().sum();
            let tokens_remaining = total_tokens.saturating_sub(tokens_taken as usize);

            // Draw stacked pyramid tokens (remaining ones)
            let stack_offset = 6.0; // Vertical offset for stacking effect
            let (stack_rect, _) = ui.allocate_exact_size(
                egui::vec2(token_size + 10.0, token_size + (tokens_remaining.saturating_sub(1)) as f32 * stack_offset + 10.0),
                egui::Sense::hover()
            );

            // Draw remaining tokens as a stack (bottom to top)
            for i in 0..tokens_remaining {
                let y_offset = (tokens_remaining - 1 - i) as f32 * stack_offset;
                let token_center = egui::pos2(
                    stack_rect.center().x,
                    stack_rect.top() + token_size / 2.0 + y_offset + 5.0
                );

                // Draw pyramid shape (triangle)
                let pyramid_height = token_size * 0.8;
                let pyramid_width = token_size * 0.7;

                let apex = egui::pos2(token_center.x, token_center.y - pyramid_height / 2.0);
                let base_left = egui::pos2(token_center.x - pyramid_width / 2.0, token_center.y + pyramid_height / 2.0);
                let base_right = egui::pos2(token_center.x + pyramid_width / 2.0, token_center.y + pyramid_height / 2.0);

                // Gold/sand colored pyramid token
                let pyramid_color = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);
                let shadow_color = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);
                let outline_color = egui::Color32::from_rgb(0x6B, 0x4A, 0x1A);

                // Draw shadow/depth on left side
                let mid_base = egui::pos2(token_center.x, token_center.y + pyramid_height / 2.0);
                ui.painter().add(egui::Shape::convex_polygon(
                    vec![apex, base_left, mid_base],
                    shadow_color,
                    egui::Stroke::NONE,
                ));

                // Draw lit side on right
                ui.painter().add(egui::Shape::convex_polygon(
                    vec![apex, mid_base, base_right],
                    pyramid_color,
                    egui::Stroke::NONE,
                ));

                // Draw outline
                ui.painter().add(egui::Shape::closed_line(
                    vec![apex, base_left, base_right],
                    egui::Stroke::new(1.5, outline_color),
                ));

                // Draw "$1" on the token
                ui.painter().text(
                    egui::pos2(token_center.x, token_center.y + 4.0),
                    egui::Align2::CENTER_CENTER,
                    "$1",
                    egui::FontId::proportional(10.0),
                    outline_color,
                );
            }

            // Show count text
            ui.add_space(4.0);
            let count_text = format!("{}/{}", tokens_remaining, total_tokens);
            ui.label(egui::RichText::new(count_text).size(12.0).color(egui::Color32::GRAY));
        });
    });

    // Left panel - Current player info and actions
    egui::SidePanel::left("player_panel").min_width(220.0).show(ctx, |ui| {
        ui.heading("Current Turn");
        ui.separator();

        let current = players.current_player();

        // Player info with color indicator
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 7.0, current_player_color);
            ui.label(egui::RichText::new(&current.name).strong().size(16.0));
        });

        ui.label(format!("Money: ${}", current.money));
        if current.is_ai {
            ui.label(egui::RichText::new("(AI Player - thinking...)").italics().color(egui::Color32::GRAY));
        }

        ui.add_space(20.0);
        ui.heading("Actions");
        ui.separator();

        // Only allow actions if not already taken this turn AND initial rolls are complete AND leg scoring is not showing
        let can_act = !turn_state.action_taken && !current.is_ai && ui_state.initial_rolls_complete && !ui_state.show_leg_scoring;

        if !ui_state.initial_rolls_complete {
            ui.label(egui::RichText::new("Setting up camels...").color(egui::Color32::YELLOW).italics());
            ui.add_space(5.0);
        } else if can_act {
            ui.label(egui::RichText::new("Choose an action:").color(egui::Color32::LIGHT_GREEN));
            ui.add_space(5.0);
        }

        ui.add_enabled_ui(can_act, |ui| {
            // Roll Pyramid button with highlight
            let roll_btn = egui::Button::new(
                egui::RichText::new("Roll Pyramid Die (+$1)")
                    .size(14.0)
            ).min_size(egui::vec2(180.0, 30.0));

            let roll_response = ui.add(roll_btn);
            if roll_response.clicked() {
                roll_action.write(RollPyramidAction);
            }
            roll_response.on_hover_text("Roll a random die from the pyramid.\nThe camel moves 1-3 spaces.\nYou earn $1.");

            ui.add_space(12.0);

            // Leg Betting Tiles - show as sophisticated cards with camel on top, value below
            ui.label(egui::RichText::new("Leg Bets:").size(12.0));
            ui.horizontal_wrapped(|ui| {
                for color in CamelColor::all() {
                    if let Some(tile) = leg_tiles.top_tile(color) {
                        let camel_color = camel_color_to_egui(color);
                        let border_color = egui::Color32::from_rgb(
                            (camel_color.r() as f32 * 0.5) as u8,
                            (camel_color.g() as f32 * 0.5) as u8,
                            (camel_color.b() as f32 * 0.5) as u8,
                        );
                        let tile_size = egui::vec2(42.0, 58.0);

                        // Create a clickable tile
                        let (rect, response) = ui.allocate_exact_size(tile_size, egui::Sense::click());

                        // Draw card background with gradient-like effect
                        // Top half: lighter (sand/cream colored for camel area)
                        let top_half = egui::Rect::from_min_max(
                            rect.min,
                            egui::pos2(rect.max.x, rect.center().y + 4.0)
                        );
                        let bottom_half = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x, rect.center().y + 4.0),
                            rect.max
                        );

                        // Card border/shadow
                        ui.painter().rect_filled(rect.expand(2.0), 5.0, egui::Color32::from_rgb(60, 50, 40));
                        ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(245, 235, 215)); // Cream/parchment background

                        // Top half - cream/parchment area for camel
                        ui.painter().rect_filled(top_half.shrink(2.0), 2.0, egui::Color32::from_rgb(250, 245, 230));

                        // Draw camel silhouette in top portion
                        let camel_rect = egui::Rect::from_min_size(
                            top_half.min + egui::vec2(4.0, 4.0),
                            egui::vec2(top_half.width() - 8.0, top_half.height() - 8.0)
                        );
                        draw_camel_silhouette(ui.painter(), camel_rect, camel_color, border_color);

                        // Bottom half - colored band with value
                        ui.painter().rect_filled(bottom_half.shrink2(egui::vec2(2.0, 0.0)), 2.0, camel_color);

                        // Draw value text in bottom portion
                        let text = format!("{}", tile.value);
                        let font_id = egui::FontId::proportional(16.0);
                        let text_color = if color == CamelColor::Yellow {
                            egui::Color32::BLACK
                        } else {
                            egui::Color32::WHITE
                        };
                        ui.painter().text(
                            bottom_half.center(),
                            egui::Align2::CENTER_CENTER,
                            &text,
                            font_id,
                            text_color,
                        );

                        // Handle click
                        if response.clicked() {
                            leg_bet_action.write(TakeLegBetAction { color });
                        }

                        // Hover effect - gold glow border
                        if response.hovered() {
                            ui.painter().rect_stroke(rect.expand(1.0), 5.0, egui::Stroke::new(3.0, egui::Color32::GOLD), egui::epaint::StrokeKind::Outside);
                        }

                        response.on_hover_text(format!("{:?} - ${}\nEarn ${} if 1st, $1 if 2nd, -$1 otherwise", color, tile.value, tile.value));
                    } else {
                        // No tile available - show empty/faded slot
                        let camel_color = camel_color_to_egui(color);
                        let faded = egui::Color32::from_rgba_unmultiplied(
                            camel_color.r(), camel_color.g(), camel_color.b(), 40
                        );
                        let tile_size = egui::vec2(42.0, 58.0);
                        let (rect, response) = ui.allocate_exact_size(tile_size, egui::Sense::hover());

                        // Faded card background
                        ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgba_unmultiplied(200, 190, 170, 60));
                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::DARK_GRAY), egui::epaint::StrokeKind::Outside);

                        // Draw faded camel silhouette
                        let top_half = egui::Rect::from_min_max(
                            rect.min,
                            egui::pos2(rect.max.x, rect.center().y + 4.0)
                        );
                        let camel_rect = egui::Rect::from_min_size(
                            top_half.min + egui::vec2(4.0, 4.0),
                            egui::vec2(top_half.width() - 8.0, top_half.height() - 8.0)
                        );
                        let faded_border = egui::Color32::from_rgba_unmultiplied(
                            (camel_color.r() as f32 * 0.5) as u8,
                            (camel_color.g() as f32 * 0.5) as u8,
                            (camel_color.b() as f32 * 0.5) as u8,
                            60
                        );
                        draw_camel_silhouette(ui.painter(), camel_rect, faded, faded_border);

                        // Draw X to indicate no tiles left
                        ui.painter().line_segment(
                            [rect.left_top() + egui::vec2(10.0, 10.0), rect.right_bottom() - egui::vec2(10.0, 10.0)],
                            egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 150))
                        );
                        ui.painter().line_segment(
                            [rect.right_top() + egui::vec2(-10.0, 10.0), rect.left_bottom() + egui::vec2(10.0, -10.0)],
                            egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 150))
                        );

                        response.on_hover_text(format!("{:?} - No tiles left", color));
                    }
                }
            });

            // Spectator Tile card (only if player has tile)
            if current.has_desert_tile {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Spectator Tile:").size(12.0));

                ui.horizontal(|ui| {
                    // Draw the desert tile card
                    let card_size = egui::vec2(50.0, 70.0);
                    let (card_rect, card_response) = ui.allocate_exact_size(card_size, egui::Sense::click());

                    // Draw the card with current flip state
                    draw_desert_tile_card(
                        ui.painter(),
                        card_rect,
                        current.character_id,
                        current_player_color,
                        ui_state.desert_tile_is_oasis,
                        ui_state.desert_tile_flip_anim,
                    );

                    // Hover effect
                    if card_response.hovered() {
                        ui.painter().rect_stroke(
                            card_rect.expand(2.0),
                            5.0,
                            egui::Stroke::new(2.0, egui::Color32::GOLD),
                            egui::epaint::StrokeKind::Outside,
                        );
                    }

                    // Click on card to place
                    if card_response.clicked() {
                        ui_state.show_desert_tile = true;
                        ui_state.desert_tile_space = None;
                    }
                    card_response.on_hover_text(format!(
                        "Click to place {} tile.\nEarn $1 when a camel lands on it.",
                        if ui_state.desert_tile_is_oasis { "Oasis (+1)" } else { "Mirage (-1)" }
                    ));

                    ui.add_space(4.0);

                    // Flip button
                    let flip_btn = egui::Button::new("⟲").min_size(egui::vec2(24.0, 24.0));
                    let flip_response = ui.add(flip_btn);
                    if flip_response.clicked() && ui_state.desert_tile_flip_anim == 0.0 {
                        // Start flip animation (will animate from 0 to 1 in update system)
                        ui_state.desert_tile_flip_anim = 0.001; // Signal to start animation
                    }
                    flip_response.on_hover_text("Flip tile to show other side");
                });
            }

            ui.add_space(8.0);

            // Race Bet button
            let race_btn = egui::Button::new(
                egui::RichText::new("Bet on Race Winner/Loser").size(14.0)
            ).min_size(egui::vec2(180.0, 30.0));

            let race_response = ui.add(race_btn);
            if race_response.clicked() {
                ui_state.show_race_betting = true;
            }
            race_response.on_hover_text("Bet on the overall race winner or loser.\nEarly correct bets earn more ($8/$5/$3/$2/$1).");
        });

        if turn_state.action_taken {
            ui.add_space(15.0);
            ui.label(egui::RichText::new("Action taken!").color(egui::Color32::YELLOW));
            ui.label("Advancing to next player...");
        }
    });

    // Right panel - All players and camel positions
    egui::SidePanel::right("players_list").min_width(280.0).show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Players");
            ui.separator();

            for (i, player) in players.players.iter().enumerate() {
                let is_current = i == players.current_player_index;
                let player_color = PLAYER_COLORS[i % PLAYER_COLORS.len()];

                // Player header with frame for current player
                let frame = if is_current {
                    egui::Frame::group(ui.style())
                        .stroke(egui::Stroke::new(2.0, egui::Color32::LIGHT_GREEN))
                        .inner_margin(4.0)
                } else {
                    egui::Frame::group(ui.style())
                        .inner_margin(4.0)
                };

                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Character avatar with colored border
                        let avatar_size = 40.0;
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(avatar_size, avatar_size), egui::Sense::hover());
                        draw_avatar(ui.painter(), rect, player.character_id, Some(player_color));

                        ui.add_space(8.0);

                        // Player info in vertical layout
                        ui.vertical(|ui| {
                            // Player name with highlight if current
                            let text = if is_current {
                                egui::RichText::new(&player.name)
                                    .strong()
                                    .size(14.0)
                                    .color(egui::Color32::WHITE)
                            } else {
                                egui::RichText::new(&player.name)
                                    .size(14.0)
                            };
                            ui.label(text);

                            // Money and AI status on second line
                            let ai_tag = if player.is_ai { " (AI)" } else { "" };
                            ui.label(egui::RichText::new(format!("${}{}", player.money, ai_tag)).size(12.0));
                        });
                    });

                    // Show player's leg bets
                    if i < player_leg_bets.bets.len() && !player_leg_bets.bets[i].is_empty() {
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            ui.label(egui::RichText::new("Leg bets:").small());
                            for bet in &player_leg_bets.bets[i] {
                                let camel_color = camel_color_to_egui(bet.camel);
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, 2.0, camel_color);
                                ui.label(egui::RichText::new(format!("{}", bet.value)).small());
                            }
                        });
                    }

                    // Show pyramid tokens earned this leg as individual icons
                    if i < player_pyramid_tokens.counts.len() && player_pyramid_tokens.counts[i] > 0 {
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            let token_count = player_pyramid_tokens.counts[i];
                            let token_size = 18.0;
                            let token_spacing = 14.0; // Slight overlap for stacked look

                            // Allocate space for all tokens
                            let total_width = token_size + (token_count.saturating_sub(1) as f32 * token_spacing);
                            let (tokens_rect, _) = ui.allocate_exact_size(
                                egui::vec2(total_width, token_size),
                                egui::Sense::hover()
                            );

                            // Draw each token
                            for t in 0..token_count {
                                let token_center = egui::pos2(
                                    tokens_rect.left() + token_size / 2.0 + (t as f32 * token_spacing),
                                    tokens_rect.center().y
                                );
                                draw_pyramid_token_icon(ui.painter(), token_center, token_size);
                            }

                            // Show total value
                            ui.label(egui::RichText::new(format!("+${}", token_count))
                                .small().color(egui::Color32::GOLD));
                        });
                    }

                    // Show race bets for this player
                    let player_id = player.id;
                    let winner_bets: Vec<_> = race_bets.winner_bets.iter()
                        .filter(|b| b.player_id == player_id)
                        .collect();
                    let loser_bets: Vec<_> = race_bets.loser_bets.iter()
                        .filter(|b| b.player_id == player_id)
                        .collect();

                    if !winner_bets.is_empty() {
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            ui.label(egui::RichText::new("Winner bets:").small().color(egui::Color32::LIGHT_GREEN));
                            for bet in winner_bets {
                                let camel_color = camel_color_to_egui(bet.camel);
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, 2.0, camel_color);
                            }
                        });
                    }

                    if !loser_bets.is_empty() {
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            ui.label(egui::RichText::new("Loser bets:").small().color(egui::Color32::from_rgb(255, 100, 100)));
                            for bet in loser_bets {
                                let camel_color = camel_color_to_egui(bet.camel);
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, 2.0, camel_color);
                            }
                        });
                    }
                });

                ui.add_space(4.0);
            }

            ui.add_space(15.0);
            ui.heading("Camel Positions");
            ui.separator();

            // Collect and sort camels by position
            let mut camel_positions: Vec<(CamelColor, u8, u8)> = camels
                .iter()
                .map(|(c, p)| (c.color, p.space_index, p.stack_position))
                .collect();
            camel_positions.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));

            // Silhouette size for camel icons
            let silhouette_size = egui::vec2(50.0, 36.0);

            for (rank, (color, space, _stack)) in camel_positions.iter().enumerate() {
                // Get animation offset for this camel
                let y_offset = camel_animations.positions
                    .iter()
                    .find(|a| a.color == *color)
                    .map(|a| a.current_y_offset)
                    .unwrap_or(0.0);

                ui.horizontal(|ui| {
                    // Rank indicator on the left with fixed width
                    let rank_text = match rank {
                        0 => "1st",
                        1 => "2nd",
                        2 => "3rd",
                        3 => "4th",
                        4 => "5th",
                        _ => "   ",
                    };
                    ui.label(egui::RichText::new(rank_text).size(16.0).strong());

                    ui.add_space(8.0);

                    // Progress bar showing position on track (space 1-16)
                    let progress = (*space as f32 + 1.0) / 16.0; // 16 spaces on track
                    let bar_width = 80.0;
                    let bar_height = 8.0;
                    let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());

                    // Draw progress bar background
                    ui.painter().rect_filled(bar_rect, 2.0, egui::Color32::from_rgb(60, 60, 60));

                    // Draw progress bar fill with camel color
                    let camel_bar_color = camel_color_to_egui(*color);
                    let fill_width = bar_width * progress;
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        egui::vec2(fill_width, bar_height),
                    );
                    ui.painter().rect_filled(fill_rect, 2.0, camel_bar_color);

                    // Draw border
                    ui.painter().rect_stroke(bar_rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)), egui::epaint::StrokeKind::Outside);

                    // Add flexible spacer to push camel to the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Draw camel silhouette on the right with animation offset
                        let camel_egui_color = camel_color_to_egui(*color);
                        let border_color = egui::Color32::from_rgb(
                            (camel_egui_color.r() as f32 * 0.5) as u8,
                            (camel_egui_color.g() as f32 * 0.5) as u8,
                            (camel_egui_color.b() as f32 * 0.5) as u8,
                        );

                        let (rect, _) = ui.allocate_exact_size(silhouette_size, egui::Sense::hover());
                        // Apply animation offset to the silhouette position
                        let animated_rect = rect.translate(egui::vec2(0.0, y_offset));
                        draw_camel_silhouette(ui.painter(), animated_rect, camel_egui_color, border_color);
                    });
                });

                // Add spacing between rows
                if rank < 4 {
                    ui.add_space(4.0);
                }
            }
        });
    });

    // Race betting popup window
    if ui_state.show_race_betting {
        egui::Window::new("Race Betting")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                let current = players.current_player();
                let current_player_index = players.current_player_index;
                let player_color = PLAYER_COLORS[current_player_index % PLAYER_COLORS.len()];
                let character_id = current.character_id;

                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new("Bet on Race Winner").size(20.0).color(egui::Color32::LIGHT_GREEN));
                    ui.label(egui::RichText::new("Choose a camel you think will WIN:").size(12.0).color(egui::Color32::GRAY));
                    ui.add_space(8.0);

                    // Winner bet cards in a row
                    ui.horizontal_wrapped(|ui| {
                        for color in CamelColor::all() {
                            let has_card = current.available_race_cards.contains(&color);
                            let card_size = egui::vec2(70.0, 90.0);
                            let (rect, response) = ui.allocate_exact_size(card_size, egui::Sense::click());

                            if has_card {
                                draw_race_bet_card(ui.painter(), rect, color, character_id, player_color, response.hovered());

                                if response.clicked() {
                                    race_bet_action.write(PlaceRaceBetAction {
                                        color,
                                        is_winner_bet: true,
                                    });
                                    ui_state.show_race_betting = false;
                                }

                                response.on_hover_text(format!("Bet on {:?} to WIN", color));
                            } else {
                                // Draw faded/unavailable card
                                draw_race_bet_card_unavailable(ui.painter(), rect, color);
                                response.on_hover_text(format!("{:?} card already used", color));
                            }
                        }
                    });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.heading(egui::RichText::new("Bet on Race Loser").size(20.0).color(egui::Color32::from_rgb(255, 100, 100)));
                    ui.label(egui::RichText::new("Choose a camel you think will LOSE:").size(12.0).color(egui::Color32::GRAY));
                    ui.add_space(8.0);

                    // Loser bet cards in a row
                    ui.horizontal_wrapped(|ui| {
                        for color in CamelColor::all() {
                            let has_card = current.available_race_cards.contains(&color);
                            let card_size = egui::vec2(70.0, 90.0);
                            let (rect, response) = ui.allocate_exact_size(card_size, egui::Sense::click());

                            if has_card {
                                draw_race_bet_card(ui.painter(), rect, color, character_id, player_color, response.hovered());

                                if response.clicked() {
                                    race_bet_action.write(PlaceRaceBetAction {
                                        color,
                                        is_winner_bet: false,
                                    });
                                    ui_state.show_race_betting = false;
                                }

                                response.on_hover_text(format!("Bet on {:?} to LOSE", color));
                            } else {
                                // Draw faded/unavailable card
                                draw_race_bet_card_unavailable(ui.painter(), rect, color);
                                response.on_hover_text(format!("{:?} card already used", color));
                            }
                        }
                    });

                    ui.add_space(20.0);

                    if ui.button(egui::RichText::new("Cancel").size(14.0)).clicked() {
                        ui_state.show_race_betting = false;
                    }
                });
            });
    }

    // Spectator tile placement popup window
    if ui_state.show_desert_tile {
        let tile_type = if ui_state.desert_tile_is_oasis { "Oasis (+1)" } else { "Mirage (-1)" };
        egui::Window::new(format!("Place Spectator Tile ({})", tile_type))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Collect spaces with camels
                let camel_spaces: std::collections::HashSet<u8> = camels
                    .iter()
                    .map(|(_, p)| p.space_index)
                    .collect();

                // Show current tile type with mini preview
                ui.horizontal(|ui| {
                    ui.label("Placing:");
                    let (preview_rect, _) = ui.allocate_exact_size(egui::vec2(36.0, 50.0), egui::Sense::hover());
                    draw_desert_tile_card(
                        ui.painter(),
                        preview_rect,
                        players.current_player().character_id,
                        current_player_color,
                        ui_state.desert_tile_is_oasis,
                        0.0,
                    );
                    ui.label(if ui_state.desert_tile_is_oasis {
                        "Camels move +1 space (on top)"
                    } else {
                        "Camels move -1 space (under)"
                    });
                });

                ui.add_space(8.0);
                ui.label("Select a space (2-16):");
                ui.label(egui::RichText::new("(Cannot place on space 1, spaces with camels, or other tiles)").small().color(egui::Color32::GRAY));
                ui.add_space(6.0);

                // Show space selection grid
                ui.horizontal_wrapped(|ui| {
                    for space in 1..TRACK_LENGTH {  // Spaces 1-15 (indices 1-15), space 0 is start
                        let has_camel = camel_spaces.contains(&space);
                        let has_tile = placed_tiles.is_space_occupied(space);
                        let is_selected = ui_state.desert_tile_space == Some(space);

                        let can_place = !has_camel && !has_tile;

                        let button_text = format!("{}", space + 1);
                        let button = if is_selected {
                            egui::Button::new(egui::RichText::new(&button_text).strong())
                                .fill(if ui_state.desert_tile_is_oasis {
                                    egui::Color32::from_rgb(80, 160, 80)
                                } else {
                                    egui::Color32::from_rgb(200, 150, 80)
                                })
                        } else {
                            egui::Button::new(&button_text)
                        };

                        ui.add_enabled_ui(can_place, |ui| {
                            if ui.add(button).clicked() {
                                ui_state.desert_tile_space = Some(space);
                            }
                        });
                    }
                });

                ui.add_space(10.0);

                if let Some(selected_space) = ui_state.desert_tile_space {
                    ui.horizontal(|ui| {
                        ui.label(format!("Selected: Space {}", selected_space + 1));
                        ui.add_space(10.0);

                        // Place button with appropriate color
                        let place_btn = egui::Button::new(
                            egui::RichText::new(format!("Place {}", tile_type)).strong()
                        ).fill(if ui_state.desert_tile_is_oasis {
                            egui::Color32::from_rgb(80, 160, 80)
                        } else {
                            egui::Color32::from_rgb(200, 150, 80)
                        });

                        if ui.add(place_btn).clicked() {
                            desert_tile_action.write(PlaceDesertTileAction {
                                space_index: selected_space,
                                is_oasis: ui_state.desert_tile_is_oasis,
                            });
                            ui_state.show_desert_tile = false;
                            ui_state.desert_tile_space = None;
                        }
                    });
                }

                ui.add_space(10.0);
                if ui.button("Cancel").clicked() {
                    ui_state.show_desert_tile = false;
                    ui_state.desert_tile_space = None;
                }
            });
    }

    // Dice result popup (enhanced with scale-in and better styling)
    if ui_state.dice_popup_timer > 0.0 {
        if let Some(ref last_roll) = ui_state.last_roll {
            // Calculate animation phases
            // Use the appropriate duration based on roll type
            let total_duration = match last_roll {
                LastRoll::Regular(_, _) => 2.0,
                LastRoll::Crazy(_, _) => 2.5,
            };
            let time_elapsed = (total_duration - ui_state.dice_popup_timer).max(0.0);

            // Scale-in effect during first 0.2 seconds
            let scale_in_duration = 0.2;
            let scale = if time_elapsed < scale_in_duration {
                // Ease-out cubic for scale-in
                let t = (time_elapsed / scale_in_duration).clamp(0.0, 1.0);
                0.5 + 0.5 * (1.0 - (1.0 - t).powi(3))
            } else {
                1.0
            };

            // Fade out during last 0.5 seconds
            let alpha = if ui_state.dice_popup_timer < 0.5 {
                ui_state.dice_popup_timer / 0.5
            } else {
                1.0
            };
            let alpha_u8 = (alpha * 255.0) as u8;

            let (text, value, camel_color, is_crazy) = match last_roll {
                LastRoll::Regular(color, value) => {
                    (format!("{:?}", color), *value, camel_color_to_egui(*color), false)
                }
                LastRoll::Crazy(color, value) => {
                    (format!("{:?}", color), *value, crazy_camel_color_to_egui(*color), true)
                }
            };

            // Create styled frame with thick colored border
            let border_color = egui::Color32::from_rgba_unmultiplied(
                camel_color.r(), camel_color.g(), camel_color.b(), alpha_u8
            );
            let bg_color = egui::Color32::from_rgba_unmultiplied(
                (camel_color.r() as f32 * 0.3) as u8,
                (camel_color.g() as f32 * 0.3) as u8,
                (camel_color.b() as f32 * 0.3) as u8,
                (alpha * 240.0) as u8,
            );

            let popup_frame = egui::Frame::new()
                .fill(bg_color)
                .stroke(egui::Stroke::new(4.0, border_color))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::symmetric(24, 16))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 4],
                    blur: 16,
                    spread: 2,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, (alpha * 100.0) as u8),
                });

            egui::Window::new("Dice Roll")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -80.0))
                .frame(popup_frame)
                .show(ctx, |ui| {
                    // Apply scale transform via spacing
                    let scaled_size = 36.0 * scale;

                    ui.vertical_centered(|ui| {
                        // Camel name
                        ui.label(
                            egui::RichText::new(&text)
                                .size(scaled_size)
                                .strong()
                                .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8)),
                        );

                        // Dice pips display
                        ui.add_space(8.0 * scale);
                        let pip_size = 8.0 * scale;
                        let pip_spacing = 12.0 * scale;
                        let box_size = 50.0 * scale;

                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(box_size, box_size),
                            egui::Sense::hover()
                        );

                        // Draw dice background
                        ui.painter().rect_filled(
                            rect,
                            8.0 * scale,
                            border_color,
                        );

                        // Draw pips based on value
                        let pip_color = if is_crazy && matches!(last_roll, LastRoll::Crazy(CrazyCamelColor::White, _)) {
                            egui::Color32::from_rgba_unmultiplied(40, 40, 40, alpha_u8)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8)
                        };

                        let center = rect.center();
                        match value {
                            1 => {
                                ui.painter().circle_filled(center, pip_size, pip_color);
                            }
                            2 => {
                                ui.painter().circle_filled(
                                    center + egui::vec2(-pip_spacing, -pip_spacing),
                                    pip_size,
                                    pip_color,
                                );
                                ui.painter().circle_filled(
                                    center + egui::vec2(pip_spacing, pip_spacing),
                                    pip_size,
                                    pip_color,
                                );
                            }
                            3 => {
                                ui.painter().circle_filled(
                                    center + egui::vec2(-pip_spacing, -pip_spacing),
                                    pip_size,
                                    pip_color,
                                );
                                ui.painter().circle_filled(center, pip_size, pip_color);
                                ui.painter().circle_filled(
                                    center + egui::vec2(pip_spacing, pip_spacing),
                                    pip_size,
                                    pip_color,
                                );
                            }
                            _ => {
                                ui.painter().circle_filled(center, pip_size, pip_color);
                            }
                        }

                        ui.add_space(8.0 * scale);

                        // Movement text
                        let move_text = if is_crazy {
                            format!("Moves {} backwards!", value)
                        } else {
                            format!("Moves {} spaces!", value)
                        };

                        ui.label(
                            egui::RichText::new(&move_text)
                                .size(scaled_size * 0.6)
                                .color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, alpha_u8)),
                        );
                    });
                });
        }
    }

}

/// System to update UI state when a regular die roll happens
pub fn update_ui_on_roll(
    mut events: MessageReader<PyramidRollResult>,
    mut ui_state: ResMut<UiState>,
) {
    for event in events.read() {
        ui_state.last_roll = Some(LastRoll::Regular(event.color, event.value));
        ui_state.dice_popup_timer = 2.0; // Show popup for 2 seconds
    }
}

/// System to update UI state when a crazy camel die roll happens
pub fn update_ui_on_crazy_roll(
    mut events: MessageReader<CrazyCamelRollResult>,
    mut ui_state: ResMut<UiState>,
) {
    for event in events.read() {
        ui_state.last_roll = Some(LastRoll::Crazy(event.color, event.value));
        ui_state.dice_popup_timer = 2.5; // Show popup for 2.5 seconds (longer for crazy camels)
    }
}

/// System to update dice popup timer
pub fn update_dice_popup_timer(
    time: Res<Time>,
    mut ui_state: ResMut<UiState>,
) {
    if ui_state.dice_popup_timer > 0.0 {
        ui_state.dice_popup_timer -= time.delta_secs();
    }

    // Update desert tile flip animation
    if ui_state.desert_tile_flip_anim > 0.0 && ui_state.desert_tile_flip_anim < 1.0 {
        // Animate the flip over 0.3 seconds
        let flip_speed = 3.33; // 1.0 / 0.3
        ui_state.desert_tile_flip_anim += time.delta_secs() * flip_speed;

        // When flip completes at 1.0, toggle the oasis state and reset animation
        if ui_state.desert_tile_flip_anim >= 1.0 {
            ui_state.desert_tile_is_oasis = !ui_state.desert_tile_is_oasis;
            ui_state.desert_tile_flip_anim = 0.0;
        }
    }
}

/// Row height constant for camel position animations
const CAMEL_POSITION_ROW_HEIGHT: f32 = 46.0; // row height including spacing
const CAMEL_POSITION_ANIMATION_SPEED: f32 = 8.0; // How fast positions animate

/// System to update camel position animations
pub fn update_camel_position_animations(
    time: Res<Time>,
    mut animations: ResMut<CamelPositionAnimations>,
    camels: Query<(&Camel, &BoardPosition)>,
) {
    // Get current camel order (sorted by position)
    let mut camel_positions: Vec<(CamelColor, u8, u8)> = camels
        .iter()
        .map(|(c, p)| (c.color, p.space_index, p.stack_position))
        .collect();
    camel_positions.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));

    let current_order: Vec<CamelColor> = camel_positions.iter().map(|(c, _, _)| *c).collect();

    // Check if order has changed
    if animations.last_order != current_order {
        // Order changed - calculate new target offsets based on position changes
        for (new_rank, &color) in current_order.iter().enumerate() {
            // Find old rank for this color
            let old_rank = animations.last_order.iter().position(|&c| c == color);

            // Find or create animation entry for this color
            let anim = animations.positions.iter_mut().find(|a| a.color == color);

            if let Some(anim) = anim {
                if let Some(old_rank) = old_rank {
                    // Set current offset to where the camel appears to be coming from
                    let rank_difference = new_rank as i32 - old_rank as i32;
                    anim.current_y_offset = -rank_difference as f32 * CAMEL_POSITION_ROW_HEIGHT;
                }
                anim.target_y_offset = 0.0;
            } else {
                // New camel - add animation entry
                let initial_offset = if let Some(old_rank) = old_rank {
                    let rank_difference = new_rank as i32 - old_rank as i32;
                    -rank_difference as f32 * CAMEL_POSITION_ROW_HEIGHT
                } else {
                    0.0
                };
                animations.positions.push(AnimatedCamelPosition {
                    color,
                    current_y_offset: initial_offset,
                    target_y_offset: 0.0,
                });
            }
        }

        animations.last_order = current_order;
    }

    // Animate offsets towards targets
    let dt = time.delta_secs();
    for anim in &mut animations.positions {
        if (anim.current_y_offset - anim.target_y_offset).abs() > 0.1 {
            let direction = if anim.current_y_offset < anim.target_y_offset { 1.0 } else { -1.0 };
            let step = CAMEL_POSITION_ANIMATION_SPEED * dt * CAMEL_POSITION_ROW_HEIGHT;
            anim.current_y_offset += direction * step;

            // Clamp to target if we overshoot
            if direction > 0.0 && anim.current_y_offset > anim.target_y_offset {
                anim.current_y_offset = anim.target_y_offset;
            } else if direction < 0.0 && anim.current_y_offset < anim.target_y_offset {
                anim.current_y_offset = anim.target_y_offset;
            }
        } else {
            anim.current_y_offset = anim.target_y_offset;
        }
    }
}

/// System to show leg scoring modal popup
pub fn leg_scoring_modal_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut players: Option<ResMut<Players>>,
    mut pyramid: Option<ResMut<Pyramid>>,
    mut leg_tiles: Option<ResMut<LegBettingTiles>>,
    mut player_leg_bets: Option<ResMut<PlayerLegBetsStore>>,
    mut player_pyramid_tokens: Option<ResMut<PlayerPyramidTokens>>,
    mut turn_state: Option<ResMut<TurnState>>,
    mut placed_tiles: Option<ResMut<PlacedDesertTiles>>,
    camels: Query<(&Camel, &BoardPosition)>,
    desert_tile_entities: Query<Entity, With<DesertTile>>,
    mut commands: Commands,
) {
    if !ui_state.show_leg_scoring {
        return;
    }

    let Some(ref mut players) = players else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Calculate scores for display
    let first_place = get_leading_camel(&camels);
    let second_place = get_second_place_camel(&camels);

    // Calculate score changes for each player
    // Structure: (name, leg_bet_total, bet_details, pyramid_tokens)
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

            // Get pyramid tokens earned this leg
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

    // Get current standings for display (INCLUDING leg earnings from this leg)
    let mut sorted_players: Vec<_> = players.players.iter()
        .enumerate()
        .map(|(idx, p)| {
            // Calculate total leg earnings for this player
            let leg_earnings = score_changes.get(idx)
                .map(|(_, leg_bet_total, _, pyramid_tokens)| {
                    *leg_bet_total + (*pyramid_tokens as i32)
                })
                .unwrap_or(0);
            // Show updated money (current + leg earnings)
            let updated_money = (p.money + leg_earnings).max(0);
            (p.name.clone(), updated_money)
        })
        .collect();
    sorted_players.sort_by(|a, b| b.1.cmp(&a.1));

    let mut should_continue = false;

    // Modal overlay
    egui::Area::new(egui::Id::new("leg_scoring_overlay"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200))
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
                        ui.heading(egui::RichText::new("Leg Complete!").size(32.0).strong());
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

                        // Show score changes for each player
                        ui.heading(egui::RichText::new("Leg Earnings").size(20.0));
                        ui.add_space(10.0);

                        for (name, leg_bet_total, details, pyramid_tokens) in &score_changes {
                            let has_bets = !details.is_empty();
                            let has_tokens = *pyramid_tokens > 0;

                            if has_bets || has_tokens {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(name).strong().size(14.0));
                                    ui.label(":");
                                    ui.add_space(8.0);

                                    // Show leg bet cards with results
                                    for (camel, value, change) in details {
                                        // Draw mini leg bet card
                                        let card_size = egui::vec2(28.0, 38.0);
                                        let (rect, _) = ui.allocate_exact_size(card_size, egui::Sense::hover());
                                        draw_mini_leg_bet_card(ui.painter(), rect, *camel, *value);

                                        // Show change next to card
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

                                    // Show pyramid tokens earned - one icon per token
                                    if *pyramid_tokens > 0 {
                                        if !details.is_empty() {
                                            ui.add_space(8.0);
                                        }
                                        let token_size = 20.0;
                                        let token_spacing = 16.0;

                                        // Allocate space for all tokens
                                        let total_width = token_size + (pyramid_tokens.saturating_sub(1) as f32 * token_spacing);
                                        let (tokens_rect, _) = ui.allocate_exact_size(
                                            egui::vec2(total_width, token_size),
                                            egui::Sense::hover()
                                        );

                                        // Draw each token
                                        for t in 0..*pyramid_tokens {
                                            let token_center = egui::pos2(
                                                tokens_rect.left() + token_size / 2.0 + (t as f32 * token_spacing),
                                                tokens_rect.center().y
                                            );
                                            draw_pyramid_token_icon(ui.painter(), token_center, token_size);
                                        }

                                        ui.label(egui::RichText::new(format!("+${}", pyramid_tokens))
                                            .size(12.0).color(egui::Color32::GOLD));
                                    }

                                    // Show total for this leg
                                    let total_leg_earnings = *leg_bet_total + (*pyramid_tokens as i32);
                                    if total_leg_earnings != 0 {
                                        ui.add_space(12.0);
                                        let total_text = if total_leg_earnings > 0 {
                                            format!("= +${}", total_leg_earnings)
                                        } else {
                                            format!("= -${}", total_leg_earnings.abs())
                                        };
                                        let total_color = if total_leg_earnings > 0 {
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

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Current standings
                        ui.heading(egui::RichText::new("Current Standings").size(20.0));
                        ui.add_space(10.0);

                        for (rank, (name, money)) in sorted_players.iter().enumerate() {
                            let rank_text = match rank {
                                0 => "1st",
                                1 => "2nd",
                                2 => "3rd",
                                3 => "4th",
                                4 => "5th",
                                5 => "6th",
                                6 => "7th",
                                7 => "8th",
                                _ => "   ",
                            };
                            ui.label(format!("{}: {} - ${}", rank_text, name, money));
                        }

                        ui.add_space(30.0);

                        let button = egui::Button::new(egui::RichText::new("Continue to Next Leg").size(18.0))
                            .min_size(egui::vec2(200.0, 50.0));
                        if ui.add(button).clicked() {
                            should_continue = true;
                        }
                    });
                });
        });

    if should_continue {
        // Apply leg bet scores
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

        // Reset for new leg
        if let Some(ref mut pyramid) = pyramid {
            pyramid.reset();
        }
        if let Some(ref mut leg_tiles) = leg_tiles {
            leg_tiles.reset();
        }
        if let Some(ref mut player_leg_bets) = player_leg_bets {
            player_leg_bets.clear_all();
        }
        if let Some(ref mut player_pyramid_tokens) = player_pyramid_tokens {
            player_pyramid_tokens.clear_all();
        }
        if let Some(ref mut turn_state) = turn_state {
            turn_state.leg_number += 1;
            turn_state.action_taken = false;
            turn_state.awaiting_action = true;
            turn_state.leg_has_started = false;
            turn_state.turn_delay_timer = 0.0;
        }

        // Clear placed desert tiles and return them to players
        if let Some(ref mut placed_tiles) = placed_tiles {
            placed_tiles.clear();
        }

        // Return desert tiles to all players
        for player in players.players.iter_mut() {
            player.has_desert_tile = true;
        }

        // Despawn visual desert tile entities
        for entity in desert_tile_entities.iter() {
            commands.entity(entity).despawn();
        }

        ui_state.show_leg_scoring = false;
    }
}
