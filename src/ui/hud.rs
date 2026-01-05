use crate::components::dice::PyramidDie;
use crate::components::{
    BoardPosition, Camel, CamelColor, CrazyCamel, CrazyCamelColor, LegBettingTiles,
    PlacedSpectatorTiles, Players, Pyramid, RaceBets, SpectatorTile, TRACK_LENGTH,
};
use crate::game::state::GameState;
use crate::systems::movement::{get_leading_camel, get_second_place_camel};
use crate::systems::setup::PendingInitialMove;
use crate::systems::turn::{
    CrazyCamelRollResult, PlaceRaceBetAction, PlaceSpectatorTileAction, PlayerLegBetsStore,
    PlayerPyramidTokens, PyramidRollResult, RollPyramidAction, TakeLegBetAction, TurnState,
};
use crate::ui::characters::{draw_avatar, CharacterId};
use crate::ui::player_setup::is_iphone;
use crate::ui::rules::{draw_rules_ui, RulesState};
use crate::ui::theme::{
    camel_color_to_egui, crazy_camel_color_to_egui, desert_button, desktop, draw_overlapping_stack,
    draw_spaced_row, layout, mobile, DesertButtonStyle, PLAYER_COLORS,
};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use bevy_egui::{egui, EguiContexts};

/// Helper function to draw a small camel silhouette for UI elements
/// Draws a stylized side-view camel using 4 layers (shadow, border, main, highlight)
/// to match the polished look of the board camels
pub fn draw_camel_silhouette(
    painter: &egui::Painter,
    rect: egui::Rect,
    color: egui::Color32,
    border_color: egui::Color32,
) {
    let center = rect.center();
    let scale = (rect.width().min(rect.height()) / 30.0).min(1.0);

    // Body - main rectangle
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

    // Head - small rectangle
    let head_width = 7.0 * scale;
    let head_height = 5.0 * scale;
    let head_center = neck_center + egui::vec2(3.0 * scale, -5.0 * scale);
    let head_rect = egui::Rect::from_center_size(head_center, egui::vec2(head_width, head_height));

    // Legs - four thin rectangles
    let leg_width = 2.5 * scale;
    let leg_height = 7.0 * scale;
    let leg_positions = [
        body_center + egui::vec2(-5.0 * scale, 7.0 * scale), // Back left
        body_center + egui::vec2(-2.0 * scale, 7.0 * scale), // Back right
        body_center + egui::vec2(4.0 * scale, 7.0 * scale),  // Front left
        body_center + egui::vec2(7.0 * scale, 7.0 * scale),  // Front right
    ];

    // === Layer 1: SHADOW ===
    let shadow_offset = egui::vec2(1.5 * scale, 1.5 * scale);
    let shadow_color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 76); // ~0.3 alpha

    painter.rect_filled(
        body_rect.translate(shadow_offset),
        1.0 * scale,
        shadow_color,
    );
    painter.rect_filled(
        hump_rect.translate(shadow_offset),
        1.0 * scale,
        shadow_color,
    );
    painter.rect_filled(
        neck_rect.translate(shadow_offset),
        0.5 * scale,
        shadow_color,
    );
    painter.rect_filled(
        head_rect.translate(shadow_offset),
        1.0 * scale,
        shadow_color,
    );
    for leg_pos in &leg_positions {
        let leg_rect = egui::Rect::from_center_size(*leg_pos, egui::vec2(leg_width, leg_height));
        painter.rect_filled(leg_rect.translate(shadow_offset), 0.5 * scale, shadow_color);
    }

    // === Layer 2: BORDER ===
    let border_expand = 1.5 * scale;
    painter.rect_filled(body_rect.expand(border_expand), 1.0 * scale, border_color);
    painter.rect_filled(hump_rect.expand(border_expand), 1.0 * scale, border_color);
    painter.rect_filled(neck_rect.expand(border_expand), 0.5 * scale, border_color);
    painter.rect_filled(head_rect.expand(border_expand), 1.0 * scale, border_color);
    for leg_pos in &leg_positions {
        let leg_rect = egui::Rect::from_center_size(*leg_pos, egui::vec2(leg_width, leg_height));
        painter.rect_filled(
            leg_rect.expand(border_expand * 0.5),
            0.5 * scale,
            border_color,
        );
    }

    // === Layer 3: MAIN COLOR ===
    painter.rect_filled(body_rect, 1.0 * scale, color);
    painter.rect_filled(hump_rect, 1.0 * scale, color);
    painter.rect_filled(neck_rect, 0.5 * scale, color);
    painter.rect_filled(head_rect, 1.0 * scale, color);
    for leg_pos in &leg_positions {
        let leg_rect = egui::Rect::from_center_size(*leg_pos, egui::vec2(leg_width, leg_height));
        painter.rect_filled(leg_rect, 0.5 * scale, color);
    }

    // === Layer 4: HIGHLIGHT ===
    let highlight_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 102); // ~0.4 alpha

    // Highlight strip on hump
    let hump_highlight_rect = egui::Rect::from_center_size(
        hump_center + egui::vec2(0.0, -2.0 * scale),
        egui::vec2((hump_width - 2.0 * scale).max(2.0), 2.0 * scale),
    );
    painter.rect_filled(hump_highlight_rect, 0.5 * scale, highlight_color);

    // Highlight strip on head
    let head_highlight_rect = egui::Rect::from_center_size(
        head_center + egui::vec2(0.0, -1.5 * scale),
        egui::vec2((head_width - 2.0 * scale).max(2.0), 1.5 * scale),
    );
    painter.rect_filled(head_highlight_rect, 0.5 * scale, highlight_color);

    // === Eye ===
    let eye_pos = head_center + egui::vec2(1.5 * scale, -0.5 * scale);
    painter.circle_filled(eye_pos, 1.0 * scale, egui::Color32::from_rgb(30, 30, 30));
}

/// Draws a grey camel with a gold crown on its head (winner icon)
pub fn draw_camel_with_crown(painter: &egui::Painter, rect: egui::Rect) {
    let camel_color = egui::Color32::from_rgb(140, 140, 140); // Grey camel
    let border_color = egui::Color32::from_rgb(80, 80, 80);

    // Draw the camel first
    draw_camel_silhouette(painter, rect, camel_color, border_color);

    // Calculate crown position based on rect (on head)
    let center = rect.center();
    let scale = (rect.width().min(rect.height()) / 30.0).min(1.0);
    let body_center = center + egui::vec2(-2.0 * scale, 2.0 * scale);
    let neck_center = body_center + egui::vec2(8.0 * scale, -4.0 * scale);
    let head_center = neck_center + egui::vec2(3.0 * scale, -5.0 * scale);

    // Crown position - on top of head
    let crown_center = head_center + egui::vec2(-1.0 * scale, -5.0 * scale);
    let crown_width = 8.0 * scale;
    let crown_height = 4.0 * scale;

    // Crown colors
    let gold = egui::Color32::from_rgb(255, 215, 0);
    let gold_dark = egui::Color32::from_rgb(200, 160, 0);

    // Crown base (rectangle)
    let base_rect = egui::Rect::from_center_size(
        crown_center + egui::vec2(0.0, 1.5 * scale),
        egui::vec2(crown_width, crown_height * 0.5),
    );
    painter.rect_filled(base_rect, 1.0 * scale, gold);

    // Crown points (3 triangles)
    let point_height = 4.0 * scale;
    let point_width = 2.5 * scale;
    let point_y = crown_center.y - 1.0 * scale;

    for i in 0..3 {
        let x_offset = (i as f32 - 1.0) * 2.5 * scale;
        let point_center = egui::pos2(crown_center.x + x_offset, point_y);

        // Triangle for crown point
        let points = vec![
            egui::pos2(point_center.x, point_center.y - point_height), // Top
            egui::pos2(point_center.x - point_width / 2.0, point_center.y), // Bottom left
            egui::pos2(point_center.x + point_width / 2.0, point_center.y), // Bottom right
        ];
        painter.add(egui::Shape::convex_polygon(
            points,
            gold,
            egui::Stroke::new(0.5 * scale, gold_dark),
        ));
    }

    // Small gems on crown points
    let gem_colors = [
        egui::Color32::from_rgb(220, 50, 50),  // Red
        egui::Color32::from_rgb(50, 100, 220), // Blue
        egui::Color32::from_rgb(220, 50, 50),  // Red
    ];
    for i in 0..3 {
        let x_offset = (i as f32 - 1.0) * 2.5 * scale;
        let gem_pos = egui::pos2(crown_center.x + x_offset, point_y - 2.0 * scale);
        painter.circle_filled(gem_pos, 1.0 * scale, gem_colors[i]);
    }
}

/// Draws a grey camel with a dunce cap on its head (loser icon)
pub fn draw_camel_with_dunce_cap(painter: &egui::Painter, rect: egui::Rect) {
    let camel_color = egui::Color32::from_rgb(140, 140, 140); // Grey camel
    let border_color = egui::Color32::from_rgb(80, 80, 80);

    // Draw the camel first
    draw_camel_silhouette(painter, rect, camel_color, border_color);

    // Draw dunce cap overlay
    draw_dunce_cap_overlay(painter, rect);
}

/// Draws just a crown overlay on top of a camel silhouette
/// The rect should be the same rect used for draw_camel_silhouette
pub fn draw_crown_overlay(painter: &egui::Painter, rect: egui::Rect) {
    // Calculate crown position based on rect (on head, right side of camel)
    let center = rect.center();
    let scale = (rect.width().min(rect.height()) / 30.0).min(1.0);
    let body_center = center + egui::vec2(-2.0 * scale, 2.0 * scale);
    let neck_center = body_center + egui::vec2(8.0 * scale, -4.0 * scale);
    let head_center = neck_center + egui::vec2(3.0 * scale, -5.0 * scale);

    // Crown position - on top of head
    let crown_center = head_center + egui::vec2(0.0, -5.0 * scale);
    let crown_width = 8.0 * scale;
    let crown_height = 4.0 * scale;

    // Crown colors
    let gold = egui::Color32::from_rgb(255, 215, 0);
    let gold_dark = egui::Color32::from_rgb(200, 160, 0);

    // Crown base (rectangle)
    let base_rect = egui::Rect::from_center_size(
        crown_center + egui::vec2(0.0, 1.5 * scale),
        egui::vec2(crown_width, crown_height * 0.5),
    );
    painter.rect_filled(base_rect, 1.0 * scale, gold);

    // Crown points (3 triangles)
    let point_height = 4.0 * scale;
    let point_width = 2.5 * scale;
    let point_y = crown_center.y - 1.0 * scale;

    for i in 0..3 {
        let x_offset = (i as f32 - 1.0) * 2.5 * scale;
        let point_center = egui::pos2(crown_center.x + x_offset, point_y);

        // Triangle for crown point
        let points = vec![
            egui::pos2(point_center.x, point_center.y - point_height), // Top
            egui::pos2(point_center.x - point_width / 2.0, point_center.y), // Bottom left
            egui::pos2(point_center.x + point_width / 2.0, point_center.y), // Bottom right
        ];
        painter.add(egui::Shape::convex_polygon(
            points,
            gold,
            egui::Stroke::new(0.5 * scale, gold_dark),
        ));
    }

    // Small gems on crown points
    let gem_colors = [
        egui::Color32::from_rgb(220, 50, 50),  // Red
        egui::Color32::from_rgb(50, 100, 220), // Blue
        egui::Color32::from_rgb(220, 50, 50),  // Red
    ];
    for i in 0..3 {
        let x_offset = (i as f32 - 1.0) * 2.5 * scale;
        let gem_pos = egui::pos2(crown_center.x + x_offset, point_y - 2.0 * scale);
        painter.circle_filled(gem_pos, 1.0 * scale, gem_colors[i]);
    }
}

/// Draws a silver crown overlay on top of a camel silhouette (2nd place)
/// The rect should be the same rect used for draw_camel_silhouette
pub fn draw_silver_crown_overlay(painter: &egui::Painter, rect: egui::Rect) {
    // Calculate crown position based on rect (on head, right side of camel)
    let center = rect.center();
    let scale = (rect.width().min(rect.height()) / 30.0).min(1.0);
    let body_center = center + egui::vec2(-2.0 * scale, 2.0 * scale);
    let neck_center = body_center + egui::vec2(8.0 * scale, -4.0 * scale);
    let head_center = neck_center + egui::vec2(3.0 * scale, -5.0 * scale);

    // Crown position - on top of head
    let crown_center = head_center + egui::vec2(0.0, -5.0 * scale);
    let crown_width = 8.0 * scale;
    let crown_height = 4.0 * scale;

    // Silver crown colors
    let silver = egui::Color32::from_rgb(200, 200, 210);
    let silver_dark = egui::Color32::from_rgb(140, 140, 150);

    // Crown base (rectangle)
    let base_rect = egui::Rect::from_center_size(
        crown_center + egui::vec2(0.0, 1.5 * scale),
        egui::vec2(crown_width, crown_height * 0.5),
    );
    painter.rect_filled(base_rect, 1.0 * scale, silver);

    // Crown points (3 triangles)
    let point_height = 4.0 * scale;
    let point_width = 2.5 * scale;
    let point_y = crown_center.y - 1.0 * scale;

    for i in 0..3 {
        let x_offset = (i as f32 - 1.0) * 2.5 * scale;
        let point_center = egui::pos2(crown_center.x + x_offset, point_y);

        // Triangle for crown point
        let points = vec![
            egui::pos2(point_center.x, point_center.y - point_height), // Top
            egui::pos2(point_center.x - point_width / 2.0, point_center.y), // Bottom left
            egui::pos2(point_center.x + point_width / 2.0, point_center.y), // Bottom right
        ];
        painter.add(egui::Shape::convex_polygon(
            points,
            silver,
            egui::Stroke::new(0.5 * scale, silver_dark),
        ));
    }

    // Small gems on crown points - light blue and pearl for silver
    let gem_colors = [
        egui::Color32::from_rgb(100, 180, 220), // Light blue
        egui::Color32::from_rgb(180, 180, 200), // Pearl
        egui::Color32::from_rgb(100, 180, 220), // Light blue
    ];
    for i in 0..3 {
        let x_offset = (i as f32 - 1.0) * 2.5 * scale;
        let gem_pos = egui::pos2(crown_center.x + x_offset, point_y - 2.0 * scale);
        painter.circle_filled(gem_pos, 1.0 * scale, gem_colors[i]);
    }
}

/// Draws just a dunce cap overlay on top of a camel silhouette
/// The rect should be the same rect used for draw_camel_silhouette
pub fn draw_dunce_cap_overlay(painter: &egui::Painter, rect: egui::Rect) {
    // Calculate cap position based on rect (on head)
    let center = rect.center();
    let scale = (rect.width().min(rect.height()) / 30.0).min(1.0);
    let body_center = center + egui::vec2(-2.0 * scale, 2.0 * scale);
    let neck_center = body_center + egui::vec2(8.0 * scale, -4.0 * scale);
    let head_center = neck_center + egui::vec2(3.0 * scale, -5.0 * scale);

    // Dunce cap position - on top of head
    let cap_base = head_center + egui::vec2(0.0, -3.0 * scale);
    let cap_height = 10.0 * scale;
    let cap_width = 6.0 * scale;

    // Cap colors - muted/subdued
    let cap_color = egui::Color32::from_rgb(100, 100, 110); // Muted grey-blue
    let cap_outline = egui::Color32::from_rgb(60, 60, 70);

    // Dunce cap triangle
    let points = vec![
        egui::pos2(cap_base.x, cap_base.y - cap_height), // Top point
        egui::pos2(cap_base.x - cap_width / 2.0, cap_base.y), // Bottom left
        egui::pos2(cap_base.x + cap_width / 2.0, cap_base.y), // Bottom right
    ];
    painter.add(egui::Shape::convex_polygon(
        points,
        cap_color,
        egui::Stroke::new(0.8 * scale, cap_outline),
    ));

    // Chin strap
    let strap_color = egui::Color32::from_rgb(70, 70, 70);
    let chin_left = head_center + egui::vec2(-3.0 * scale, 2.0 * scale);
    let chin_right = head_center + egui::vec2(3.0 * scale, 2.0 * scale);
    painter.line_segment(
        [chin_left, chin_right],
        egui::Stroke::new(0.5 * scale, strap_color),
    );
}

/// Indicates what type of race bet was placed for displaying on unavailable cards
#[derive(Clone, Copy)]
enum PlacedBetType {
    Winner,
    Loser,
}

/// Helper function to draw a mini leg bet card (camel silhouette on top, value on bottom)
pub fn draw_mini_leg_bet_card(
    painter: &egui::Painter,
    rect: egui::Rect,
    camel_color: CamelColor,
    value: u8,
) {
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
    let top_half =
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.center().y + 2.0));
    let bottom_half =
        egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.center().y + 2.0), rect.max);

    // Top half - cream for camel
    painter.rect_filled(
        top_half.shrink(1.0),
        1.0,
        egui::Color32::from_rgb(250, 245, 230),
    );

    // Draw camel silhouette
    let camel_rect = egui::Rect::from_min_size(
        top_half.min + egui::vec2(2.0, 2.0),
        egui::vec2(top_half.width() - 4.0, top_half.height() - 4.0),
    );
    draw_camel_silhouette(painter, camel_rect, color, border_color);

    // Bottom half - cream background with gold coin
    painter.rect_filled(
        bottom_half.shrink(1.0),
        1.0,
        egui::Color32::from_rgb(250, 245, 230),
    );

    // Draw gold coin with value
    let coin_center = bottom_half.center();
    let coin_radius = (bottom_half.height() * 0.38).min(bottom_half.width() * 0.38);

    // Gold colors (matching pyramid token)
    let gold_light = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);
    let gold_dark = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);
    let gold_outline = egui::Color32::from_rgb(0x6B, 0x4A, 0x1A);

    // Outer shadow/depth
    painter.circle_filled(
        coin_center + egui::vec2(1.0, 1.0),
        coin_radius,
        gold_outline,
    );
    // Main coin body
    painter.circle_filled(coin_center, coin_radius, gold_light);
    // Inner shadow ring for depth
    painter.circle_stroke(
        coin_center,
        coin_radius * 0.85,
        egui::Stroke::new(1.0, gold_dark),
    );
    // Outer edge
    painter.circle_stroke(
        coin_center,
        coin_radius,
        egui::Stroke::new(1.5, gold_outline),
    );

    // Value text on coin
    painter.text(
        coin_center,
        egui::Align2::CENTER_CENTER,
        format!("{}", value),
        egui::FontId::proportional(coin_radius * 0.9),
        gold_outline,
    );
}

/// Helper function to draw a tiny leg bet indicator for player's bet collection
/// Smaller than draw_mini_leg_bet_card, designed for overlapping display
pub fn draw_mini_leg_bet_indicator(
    painter: &egui::Painter,
    rect: egui::Rect,
    camel_color: CamelColor,
    value: u8,
) {
    let color = camel_color_to_egui(camel_color);

    // Card border/shadow
    painter.rect_filled(rect.expand(0.5), 1.5, egui::Color32::from_rgb(50, 40, 35));
    painter.rect_filled(rect, 1.0, egui::Color32::from_rgb(245, 235, 215));

    // Top half - camel color block
    let top_half = egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.center().y));
    painter.rect_filled(top_half.shrink(0.5), 0.5, color);

    // Bottom half - value
    let bottom_center = egui::pos2(rect.center().x, rect.max.y - rect.height() * 0.25);
    painter.text(
        bottom_center,
        egui::Align2::CENTER_CENTER,
        format!("{}", value),
        egui::FontId::proportional(rect.height() * 0.35),
        egui::Color32::from_rgb(100, 70, 30),
    );
}

/// Helper function to draw a small pyramid token icon
/// Returns the rect used for the icon
pub fn draw_pyramid_token_icon(painter: &egui::Painter, center: egui::Pos2, size: f32) {
    let pyramid_height = size * 0.8;
    let pyramid_width = size * 0.7;

    let apex = egui::pos2(center.x, center.y - pyramid_height / 2.0);
    let base_left = egui::pos2(
        center.x - pyramid_width / 2.0,
        center.y + pyramid_height / 2.0,
    );
    let base_right = egui::pos2(
        center.x + pyramid_width / 2.0,
        center.y + pyramid_height / 2.0,
    );
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

/// Draw an interactive pyramid button with optional flip animation
/// flip_progress: 0.0 = not animating, 0.01-1.0 = flip in progress
pub fn draw_pyramid_button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    flip_progress: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let painter = ui.painter();

    // Pyramid colors
    let pyramid_light = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);
    let pyramid_dark = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);
    let outline_color = egui::Color32::from_rgb(0x6B, 0x4A, 0x1A);

    // Hover effect - brighten colors
    let (light, dark) = if response.hovered() {
        (
            egui::Color32::from_rgb(0xE4, 0xB8, 0x5B),
            egui::Color32::from_rgb(0xB0, 0x8A, 0x40),
        )
    } else {
        (pyramid_light, pyramid_dark)
    };

    // Calculate rotation angle for shake (oscillates with decay)
    let rotation_angle = if flip_progress > 0.0 {
        let shake_intensity = 0.15; // radians max rotation (~8.5 degrees)
        let shake_frequency = 4.0; // number of full oscillations
        let decay = 1.0 - flip_progress; // decreases over time
        (flip_progress * shake_frequency * std::f32::consts::TAU).sin() * shake_intensity * decay
    } else {
        0.0
    };

    // Helper to rotate a point around a center
    let rotate_point = |point: egui::Pos2, center: egui::Pos2, angle: f32| -> egui::Pos2 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let dx = point.x - center.x;
        let dy = point.y - center.y;
        egui::pos2(
            center.x + dx * cos_a - dy * sin_a,
            center.y + dx * sin_a + dy * cos_a,
        )
    };

    // Define pyramid points (unrotated)
    let center = rect.center();
    let apex_base = egui::pos2(rect.center().x, rect.top() + 4.0);
    let base_left_base = egui::pos2(rect.left() + 4.0, rect.bottom() - 4.0);
    let base_right_base = egui::pos2(rect.right() - 4.0, rect.bottom() - 4.0);
    let mid_base_base = egui::pos2(rect.center().x, rect.bottom() - 4.0);

    // Apply rotation
    let apex = rotate_point(apex_base, center, rotation_angle);
    let base_left = rotate_point(base_left_base, center, rotation_angle);
    let base_right = rotate_point(base_right_base, center, rotation_angle);
    let mid_base = rotate_point(mid_base_base, center, rotation_angle);

    // Left (shadow) side
    painter.add(egui::Shape::convex_polygon(
        vec![apex, base_left, mid_base],
        dark,
        egui::Stroke::NONE,
    ));

    // Right (lit) side
    painter.add(egui::Shape::convex_polygon(
        vec![apex, mid_base, base_right],
        light,
        egui::Stroke::NONE,
    ));

    // Outline
    painter.add(egui::Shape::closed_line(
        vec![apex, base_left, base_right],
        egui::Stroke::new(1.5, outline_color),
    ));

    // Draw "Roll" text and "+$1" (rotate with pyramid)
    let roll_pos_base = egui::pos2(center.x, center.y - 4.0);
    let cost_pos_base = egui::pos2(center.x, center.y + size.y * 0.2);
    let roll_pos = rotate_point(roll_pos_base, center, rotation_angle);
    let cost_pos = rotate_point(cost_pos_base, center, rotation_angle);

    painter.text(
        roll_pos,
        egui::Align2::CENTER_CENTER,
        "Roll",
        egui::FontId::proportional(size.y * 0.18),
        outline_color,
    );
    painter.text(
        cost_pos,
        egui::Align2::CENTER_CENTER,
        "+$1",
        egui::FontId::proportional(size.y * 0.14),
        outline_color,
    );

    response
}

/// Helper function to draw a spectator tile card with player avatar on top and +1/-1 on bottom
/// flip_progress: 0.0 = front fully visible, 1.0 = back fully visible
/// Uses clip-rect approach for unified flip effect - all elements clipped identically
pub fn draw_spectator_tile_card(
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
    if scale_x < 0.02 {
        return;
    }

    // Determine which side to show based on is_oasis and show_front
    let showing_oasis = if show_front { is_oasis } else { !is_oasis };

    // Create clip rect - a narrowing horizontal strip centered on the card
    // This creates a uniform flip effect where all elements are clipped identically
    let center_x = rect.center().x;
    let visible_half_width = rect.width() * scale_x / 2.0;
    let clip_rect = egui::Rect::from_x_y_ranges(
        center_x - visible_half_width..=center_x + visible_half_width,
        rect.top()..=rect.bottom(),
    );

    // Create a clipped painter - all drawing will be clipped to the narrowing strip
    let clipped_painter = painter.with_clip_rect(clip_rect);

    // Draw full card content at normal size - clipping creates the flip effect
    draw_spectator_tile_content(
        &clipped_painter,
        rect,
        character_id,
        player_color,
        showing_oasis,
    );
}

/// Helper function to draw spectator tile card content at full size (used with clipping for flip effect)
fn draw_spectator_tile_content(
    painter: &egui::Painter,
    rect: egui::Rect,
    character_id: CharacterId,
    player_color: egui::Color32,
    showing_oasis: bool,
) {
    // Colors for each side
    let (bg_color, _value_color, value_text) = if showing_oasis {
        // Oasis (+1): Green/lush colors
        (
            egui::Color32::from_rgb(80, 160, 80), // Green
            egui::Color32::from_rgb(50, 120, 50), // Dark green
            "+1",
        )
    } else {
        // Mirage (-1): Sandy/orange colors
        (
            egui::Color32::from_rgb(200, 150, 80), // Sandy
            egui::Color32::from_rgb(160, 100, 40), // Dark sandy
            "-1",
        )
    };

    // Card shadow
    painter.rect_filled(rect.expand(2.0), 5.0, egui::Color32::from_rgb(40, 30, 20));

    // Card background
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(245, 235, 215));

    // Split into top (avatar) and bottom (value)
    let top_half =
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.center().y + 4.0));
    let bottom_half =
        egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.center().y + 4.0), rect.max);

    // Top half - cream background with avatar
    painter.rect_filled(
        top_half.shrink(2.0),
        2.0,
        egui::Color32::from_rgb(250, 245, 230),
    );

    // Draw player avatar in top portion at full size
    let avatar_size = (top_half.height() - 8.0).min(top_half.width() - 8.0);
    let avatar_rect =
        egui::Rect::from_center_size(top_half.center(), egui::vec2(avatar_size, avatar_size));
    draw_avatar(painter, avatar_rect, character_id, Some(player_color));

    // Bottom half - colored band with value
    painter.rect_filled(bottom_half.shrink2(egui::vec2(2.0, 0.0)), 2.0, bg_color);

    // Draw +1 or -1 text (slightly left of center to make room for coin)
    let text_offset = -rect.width() * 0.12;
    painter.text(
        bottom_half.center() + egui::vec2(text_offset, 0.0),
        egui::Align2::CENTER_CENTER,
        value_text,
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );

    // Draw gold coin with "1" in top-right corner of bottom half (shows $1 reward for landing)
    let coin_radius = 7.0;
    let coin_center = egui::pos2(
        bottom_half.right() - coin_radius - 4.0,
        bottom_half.top() + coin_radius + 2.0,
    );

    // Gold coin colors
    let coin_gold = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);
    let coin_dark = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);

    // Coin circle with darker border
    painter.circle_filled(coin_center, coin_radius, coin_gold);
    painter.circle_stroke(coin_center, coin_radius, egui::Stroke::new(1.0, coin_dark));

    // "1" text on coin
    painter.text(
        coin_center,
        egui::Align2::CENTER_CENTER,
        "1",
        egui::FontId::proportional(9.0),
        coin_dark,
    );
}

/// Draw a flip/sync icon (two curved arrows) for the spectator tile flip button
fn draw_flip_icon(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    let r = size * 0.38; // Radius of the circular path
    let stroke = egui::Stroke::new(size * 0.12, color);
    let arrow_size = size * 0.15;

    // Draw using line segments to approximate arcs
    // Top arc (right half of circle, pointing right)
    let segments = 8;
    for i in 0..segments {
        let angle1 =
            std::f32::consts::PI * 0.15 + (i as f32 / segments as f32) * std::f32::consts::PI * 0.7;
        let angle2 = std::f32::consts::PI * 0.15
            + ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 0.7;
        let p1 = center + egui::vec2(r * angle1.cos(), -r * angle1.sin());
        let p2 = center + egui::vec2(r * angle2.cos(), -r * angle2.sin());
        painter.line_segment([p1, p2], stroke);
    }

    // Bottom arc (left half of circle, pointing left)
    for i in 0..segments {
        let angle1 =
            std::f32::consts::PI * 1.15 + (i as f32 / segments as f32) * std::f32::consts::PI * 0.7;
        let angle2 = std::f32::consts::PI * 1.15
            + ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 0.7;
        let p1 = center + egui::vec2(r * angle1.cos(), -r * angle1.sin());
        let p2 = center + egui::vec2(r * angle2.cos(), -r * angle2.sin());
        painter.line_segment([p1, p2], stroke);
    }

    // Arrow head on top arc (pointing right/down)
    let top_arrow_pos = center + egui::vec2(r * 0.85, -r * 0.5);
    painter.line_segment(
        [
            top_arrow_pos,
            top_arrow_pos + egui::vec2(-arrow_size, -arrow_size * 0.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            top_arrow_pos,
            top_arrow_pos + egui::vec2(-arrow_size * 0.3, arrow_size),
        ],
        stroke,
    );

    // Arrow head on bottom arc (pointing left/up)
    let bottom_arrow_pos = center + egui::vec2(-r * 0.85, r * 0.5);
    painter.line_segment(
        [
            bottom_arrow_pos,
            bottom_arrow_pos + egui::vec2(arrow_size, arrow_size * 0.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            bottom_arrow_pos,
            bottom_arrow_pos + egui::vec2(arrow_size * 0.3, -arrow_size),
        ],
        stroke,
    );

    // Small center circle (eye/viewer symbol)
    painter.circle_filled(center, size * 0.12, color);
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
    painter.rect_filled(
        shadow_rect,
        6.0,
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60),
    );

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
        painter.rect_stroke(
            rect.expand(3.0),
            6.0,
            egui::Stroke::new(3.0, egui::Color32::GOLD),
            egui::epaint::StrokeKind::Outside,
        );
    }
}

/// Helper function to draw an unavailable/used race bet card
/// Shows a camel with crown (winner bet) or dunce cap (loser bet) instead of an X
fn draw_race_bet_card_unavailable(
    painter: &egui::Painter,
    rect: egui::Rect,
    camel_color: CamelColor,
    placed_bet: PlacedBetType,
) {
    let color = camel_color_to_egui(camel_color);
    let faded_color = egui::Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        100, // Slightly more visible than before since we're showing content
    );
    let border_color = egui::Color32::from_rgba_unmultiplied(
        (color.r() as f32 * 0.5) as u8,
        (color.g() as f32 * 0.5) as u8,
        (color.b() as f32 * 0.5) as u8,
        120,
    );

    // Faded card border
    painter.rect_filled(rect.expand(2.0), 6.0, border_color);

    // Faded card background
    painter.rect_filled(rect, 5.0, faded_color);

    // Draw camel silhouette with crown or dunce cap based on bet type
    let icon_rect = egui::Rect::from_center_size(
        rect.center() + egui::vec2(0.0, -5.0), // Shift up slightly to make room for label
        egui::vec2(rect.width() * 0.7, rect.height() * 0.55),
    );

    // Use the camel's actual color (not grey) so player can see which color they bet on
    let camel_border = egui::Color32::from_rgb(
        (color.r() as f32 * 0.6) as u8,
        (color.g() as f32 * 0.6) as u8,
        (color.b() as f32 * 0.6) as u8,
    );
    draw_camel_silhouette(painter, icon_rect, color, camel_border);

    // Draw the appropriate accessory based on bet type
    match placed_bet {
        PlacedBetType::Winner => draw_crown_overlay(painter, icon_rect),
        PlacedBetType::Loser => draw_dunce_cap_overlay(painter, icon_rect),
    }

    // Camel name at the bottom
    let text_color = egui::Color32::from_rgba_unmultiplied(80, 80, 80, 200);
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
    pub die_color: Option<CamelColor>, // None = crazy die (gray)
    pub start_time: f64,               // When animation started (seconds)
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
    pub start_pos: egui::Pos2, // Card stack position (screen coords)
    pub end_pos: egui::Pos2,   // Player panel edge
    pub start_time: f64,
    pub phase: CardFlightPhase,
}

/// UI state for showing different panels
#[derive(Resource)]
pub struct UiState {
    pub show_winner_betting: bool, // Show winner bet modal
    pub show_loser_betting: bool,  // Show loser bet modal
    pub show_spectator_tile: bool,
    pub spectator_tile_space: Option<u8>, // Selected space for spectator tile
    pub spectator_tile_is_oasis: bool,    // Current side of spectator tile card (true = oasis +1)
    pub spectator_tile_flip_anim: f32,    // Animation progress for card flip (0.0 to 1.0)
    pub spectator_tile_selected: bool, // Whether spectator tile card is selected for placement (mobile)
    pub last_roll: Option<LastRoll>,
    pub dice_popup_delay: f32, // Delay before showing popup (waits for shake animation)
    pub dice_popup_timer: f32, // Timer for dice result popup fade
    pub show_leg_scoring: bool, // Show leg scoring modal
    pub leg_scoring_delay: f32, // Delay timer before showing leg scoring modal (800ms)
    pub game_end_delay: f32,   // Delay timer before transitioning to GameEnd state (800ms)
    pub show_rules: bool,      // Show game rules modal
    pub initial_rolls_complete: bool, // Whether initial setup rolls have finished
    pub exit_fullscreen_requested: bool, // Request to exit fullscreen mode
    pub enter_fullscreen_requested: bool, // Request to enter fullscreen mode
    pub use_side_panels: bool, // Layout mode: true = side panels (landscape), false = top/bottom (portrait)
    pub game_board_rect: Option<egui::Rect>, // Measured game board area from CentralPanel
    #[allow(dead_code)]
    pub mobile_tab: MobileTab, // Current tab in mobile view (deprecated)
    pub die_roll_animation: Option<DieRollAnimation>, // Animation for die being selected/rolled
    pub pyramid_flip_anim: f32, // 0.0 = not animating, 0.01-1.0 = flip in progress
    pub card_flight_animation: Option<CardFlightAnimation>, // Animation for leg bet card flying to player
    pub leg_bet_card_positions: [Option<egui::Pos2>; 5], // Screen positions of leg bet card stacks (indexed by CamelColor)
    pub player_bet_area_pos: Option<egui::Pos2>, // Screen position where player's bets are displayed
    pub show_debug_overlay: bool,                // Show debug overlay with window dimensions
}

/// Animated position entry for the camel positions panel
#[derive(Clone, Copy)]
pub struct AnimatedCamelPosition {
    pub color: CamelColor,
    pub current_y_offset: f32, // Current Y position offset for horizontal slide animation
    pub target_y_offset: f32,  // Target Y position (0 = at rank position)
    pub current_podium_y: f32, // Current vertical offset for podium (negative = up)
    pub target_podium_y: f32,  // Target vertical offset for podium
}

/// Resource for tracking camel position animations in UI
#[derive(Resource, Default)]
pub struct CamelPositionAnimations {
    pub positions: Vec<AnimatedCamelPosition>,
    pub last_order: Vec<CamelColor>, // Previous frame's order for detecting changes
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_winner_betting: false,
            show_loser_betting: false,
            show_spectator_tile: false,
            spectator_tile_space: None,
            spectator_tile_is_oasis: true, // Start with oasis side (+1)
            spectator_tile_flip_anim: 0.0,
            spectator_tile_selected: false,
            last_roll: None,
            dice_popup_delay: 0.0,
            dice_popup_timer: 0.0,
            show_leg_scoring: false,
            leg_scoring_delay: 0.0,
            game_end_delay: 0.0,
            show_rules: false,
            initial_rolls_complete: false,
            exit_fullscreen_requested: false,
            enter_fullscreen_requested: false,
            use_side_panels: true, // Default to side panels (landscape)
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

pub fn game_hud_ui(
    mut contexts: EguiContexts,
    game_resources: (
        Option<Res<Players>>,
        Option<Res<Pyramid>>,
        Option<Res<LegBettingTiles>>,
        Option<Res<TurnState>>,
        Option<Res<PlacedSpectatorTiles>>,
        Option<Res<PlayerLegBetsStore>>,
        Option<Res<PlayerPyramidTokens>>,
        Option<Res<RaceBets>>,
    ),
    mut ui_state: ResMut<UiState>,
    mut rules_state: ResMut<RulesState>,
    camel_animations: Res<CamelPositionAnimations>,
    actions: (
        MessageWriter<RollPyramidAction>,
        MessageWriter<TakeLegBetAction>,
        MessageWriter<PlaceRaceBetAction>,
        MessageWriter<PlaceSpectatorTileAction>,
    ),
    camels: Query<(&Camel, &BoardPosition), Without<PendingInitialMove>>,
    crazy_camels: Query<(&CrazyCamel, &BoardPosition), Without<PendingInitialMove>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut windows: Query<&mut Window>,
    time: Res<Time>,
    mut initial_rolls: Option<ResMut<crate::systems::setup::InitialSetupRolls>>,
) {
    let (
        players,
        pyramid,
        leg_tiles,
        turn_state,
        placed_tiles,
        player_leg_bets,
        player_pyramid_tokens,
        race_bets,
    ) = game_resources;
    let (mut roll_action, mut leg_bet_action, mut race_bet_action, mut spectator_tile_action) =
        actions;
    let Some(players) = players else { return };
    let Some(pyramid) = pyramid else { return };
    let Some(leg_tiles) = leg_tiles else { return };
    let Some(turn_state) = turn_state else { return };
    let Some(placed_tiles) = placed_tiles else {
        return;
    };
    let Some(player_leg_bets) = player_leg_bets else {
        return;
    };
    let Some(player_pyramid_tokens) = player_pyramid_tokens else {
        return;
    };
    let Some(race_bets) = race_bets else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Draw rules UI if triggered from HUD
    if ui_state.show_rules {
        rules_state.is_open = true;
        ui_state.show_rules = false;
    }
    draw_rules_ui(
        ctx,
        &mut rules_state,
        !ui_state.use_side_panels,
        time.delta_secs(),
    );

    // Debug overlay - show window dimensions in top left (only in debug builds)
    #[cfg(debug_assertions)]
    if ui_state.show_debug_overlay {
        egui::Area::new(egui::Id::new("debug_dimensions_overlay"))
            .fixed_pos(egui::pos2(8.0, 8.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Ok(window) = windows.single() {
                        let text = format!("{}x{}", window.width() as u32, window.height() as u32);
                        ui.label(
                            egui::RichText::new(text)
                                .size(12.0)
                                .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180))
                                .background_color(egui::Color32::from_rgba_unmultiplied(
                                    0, 0, 0, 120,
                                )),
                        );
                    }
                });
            });
    }

    // Shared current player color (used in multiple places)
    let current_player_color =
        PLAYER_COLORS[players.current_player().color_index % PLAYER_COLORS.len()];

    // Top bar - Game info (responsive based on layout mode)
    egui::TopBottomPanel::top("game_info").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if !ui_state.use_side_panels {
                // Portrait/compact layout - minimal header
                ui.label(egui::RichText::new("Camel Up").strong().size(14.0));
                ui.separator();
                ui.label(egui::RichText::new(format!("Leg {}", turn_state.leg_number)).size(12.0));
            } else {
                // Desktop: full layout
                ui.heading("Camel Up");
                ui.separator();
                ui.label(format!("Leg {}", turn_state.leg_number));
                ui.separator();

                // Roll tokens - show remaining dice as dice icons (rounded squares)
                ui.label("Roll Tokens:");
                let token_size = 18.0;
                let overlap = 5.0;
                let rounding = 3.0;
                let current_time = time.elapsed_secs_f64();
                for die in pyramid.dice.iter() {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(token_size - overlap, token_size),
                        egui::Sense::hover(),
                    );
                    let die_rect = egui::Rect::from_center_size(
                        rect.center(),
                        egui::vec2(token_size - 2.0, token_size - 2.0),
                    );
                    match die {
                        PyramidDie::Regular(regular) => {
                            let color = camel_color_to_egui(regular.color);
                            ui.painter().rect_filled(die_rect, rounding, color);
                            ui.painter().rect_stroke(
                                die_rect,
                                rounding,
                                egui::Stroke::new(1.0, egui::Color32::BLACK),
                                egui::epaint::StrokeKind::Outside,
                            );
                        }
                        PyramidDie::Crazy { .. } => {
                            ui.painter().rect_filled(
                                die_rect,
                                rounding,
                                egui::Color32::from_rgb(100, 100, 100),
                            );
                            ui.painter().rect_stroke(
                                die_rect,
                                rounding,
                                egui::Stroke::new(1.0, egui::Color32::BLACK),
                                egui::epaint::StrokeKind::Outside,
                            );
                        }
                    }
                }
                // Render animating die (the one being rolled)
                if let Some(ref anim) = ui_state.die_roll_animation {
                    let elapsed = (current_time - anim.start_time) as f32;
                    let anim_duration = 0.4; // Total animation duration
                    if elapsed < anim_duration {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(token_size - overlap, token_size),
                            egui::Sense::hover(),
                        );
                        // Shake phase (0-0.2s), then drop phase (0.2-0.4s)
                        let (shake_offset, drop_offset, scale, alpha) = if elapsed < 0.2 {
                            // Shake phase
                            let shake = (elapsed * 50.0).sin() * 4.0;
                            (shake, 0.0, 1.0, 1.0)
                        } else {
                            // Drop phase
                            let drop_progress = (elapsed - 0.2) / 0.2;
                            (
                                0.0,
                                drop_progress * 20.0,
                                1.0 - drop_progress * 0.3,
                                1.0 - drop_progress,
                            )
                        };
                        let size = (token_size - 2.0) * scale;
                        let center = rect.center() + egui::vec2(shake_offset, drop_offset);
                        let die_rect = egui::Rect::from_center_size(center, egui::vec2(size, size));
                        let base_color = match anim.die_color {
                            Some(color) => camel_color_to_egui(color),
                            None => egui::Color32::from_rgb(100, 100, 100), // Crazy die
                        };
                        let color = egui::Color32::from_rgba_unmultiplied(
                            base_color.r(),
                            base_color.g(),
                            base_color.b(),
                            (255.0 * alpha) as u8,
                        );
                        ui.painter().rect_filled(die_rect, rounding * scale, color);
                        ui.painter().rect_stroke(
                            die_rect,
                            rounding * scale,
                            egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_unmultiplied(
                                    0,
                                    0,
                                    0,
                                    (255.0 * alpha) as u8,
                                ),
                            ),
                            egui::epaint::StrokeKind::Outside,
                        );
                    }
                }
                ui.add_space(4.0);

                if let Some(ref last_roll) = ui_state.last_roll {
                    ui.separator();
                    match last_roll {
                        LastRoll::Regular(color, value) => {
                            ui.label(format!("Last roll: {:?} moved {} spaces", color, value));
                        }
                        LastRoll::Crazy(color, value) => {
                            ui.label(format!(
                                "Last roll: {:?} crazy camel moved {} backwards!",
                                color, value
                            ));
                        }
                    }
                }
            }

            // Add flexible spacer to push buttons to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let small_style = DesertButtonStyle::small();
                let compact_style = DesertButtonStyle::compact();

                if !ui_state.use_side_panels {
                    // Portrait: smaller buttons
                    if desert_button(ui, "Menu", &small_style).clicked() {
                        next_state.set(GameState::MainMenu);
                    }
                    if desert_button(ui, "?", &compact_style)
                        .on_hover_text("How to Play")
                        .clicked()
                    {
                        ui_state.show_rules = true;
                    }
                } else {
                    if desert_button(ui, "Back to Menu", &small_style).clicked() {
                        next_state.set(GameState::MainMenu);
                    }
                    if desert_button(ui, "?", &compact_style)
                        .on_hover_text("How to Play")
                        .clicked()
                    {
                        ui_state.show_rules = true;
                    }
                }

                // Show fullscreen toggle button (hidden on iPhone where Fullscreen API is unsupported)
                if !is_iphone() {
                    if let Ok(window) = windows.single() {
                        let is_fullscreen = window.mode != WindowMode::Windowed;
                        if desert_button(ui, "⛶", &compact_style).clicked() {
                            ui_state.exit_fullscreen_requested = is_fullscreen;
                        }
                    }
                }

                // Debug overlay toggle button (only in debug builds)
                #[cfg(debug_assertions)]
                {
                    let debug_icon = "#";
                    if desert_button(ui, debug_icon, &compact_style)
                        .on_hover_text("Toggle debug overlay")
                        .clicked()
                    {
                        ui_state.show_debug_overlay = !ui_state.show_debug_overlay;
                    }
                }
            });
        });
    });

    // Process fullscreen requests
    if ui_state.exit_fullscreen_requested {
        ui_state.exit_fullscreen_requested = false;
        if let Ok(mut window) = windows.single_mut() {
            window.mode = WindowMode::Windowed;
        }
    }
    if ui_state.enter_fullscreen_requested {
        ui_state.enter_fullscreen_requested = false;
        if let Ok(mut window) = windows.single_mut() {
            window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
        }
    }

    // Branch based on layout mode (side panels vs top/bottom)
    if ui_state.use_side_panels {
        // Landscape layout - side panels
        render_desktop_ui(
            ctx,
            &players,
            &pyramid,
            &leg_tiles,
            &turn_state,
            &*placed_tiles,
            &player_leg_bets,
            &player_pyramid_tokens,
            &race_bets,
            &mut ui_state,
            &camel_animations,
            &mut roll_action,
            &mut leg_bet_action,
            &mut race_bet_action,
            &mut spectator_tile_action,
            &camels,
            current_player_color,
            &mut initial_rolls,
        );
    } else {
        // Portrait layout - top/bottom panels
        render_mobile_ui(
            ctx,
            &players,
            &pyramid,
            &leg_tiles,
            &turn_state,
            &*placed_tiles,
            &player_leg_bets,
            &player_pyramid_tokens,
            &race_bets,
            &mut ui_state,
            &camel_animations,
            &mut roll_action,
            &mut leg_bet_action,
            &mut race_bet_action,
            &mut spectator_tile_action,
            &camels,
            current_player_color,
            &mut initial_rolls,
        );
    }

    // Measure remaining space for game board using CentralPanel
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            ui_state.game_board_rect = Some(ui.available_rect_before_wrap());
        });

    // Shared popup windows (race betting, spectator tile placement, dice result)
    render_popup_windows(
        ctx,
        &*players,
        &*placed_tiles,
        &*race_bets,
        &mut *ui_state,
        &mut race_bet_action,
        &mut spectator_tile_action,
        &camels,
        &crazy_camels,
        current_player_color,
    );

    // Card flight animation overlay (drawn on top of everything)
    render_card_flight_animation(ctx, &mut *ui_state, time.elapsed_secs_f64());
}

/// Render the flying card animation overlay
fn render_card_flight_animation(ctx: &egui::Context, ui_state: &mut UiState, current_time: f64) {
    if let Some(ref mut anim) = ui_state.card_flight_animation {
        let elapsed = (current_time - anim.start_time) as f32;

        // Animation timing (0.4s total)
        const FLY_DURATION: f32 = 0.25;
        const DISAPPEAR_DURATION: f32 = 0.07;
        const REAPPEAR_DURATION: f32 = 0.08;
        const TOTAL_DURATION: f32 = FLY_DURATION + DISAPPEAR_DURATION + REAPPEAR_DURATION;

        // Update phase based on elapsed time
        let phase = if elapsed < FLY_DURATION {
            CardFlightPhase::FlyingToPanel
        } else if elapsed < FLY_DURATION + DISAPPEAR_DURATION {
            CardFlightPhase::DisappearingUnder
        } else if elapsed < TOTAL_DURATION {
            CardFlightPhase::ReappearingInside
        } else {
            CardFlightPhase::Done
        };
        anim.phase = phase;

        // Get the color for the card
        let color = camel_color_to_egui(anim.color);
        let card_width = 36.0;
        let card_height = 48.0;

        // Draw based on phase
        egui::Area::new(egui::Id::new("card_flight_overlay"))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let painter = ui.painter();

                match phase {
                    CardFlightPhase::FlyingToPanel => {
                        // Interpolate position with ease-out
                        let t = elapsed / FLY_DURATION;
                        let ease_t = 1.0 - (1.0 - t).powi(3); // Cubic ease-out
                        let pos = egui::pos2(
                            anim.start_pos.x + (anim.end_pos.x - anim.start_pos.x) * ease_t,
                            anim.start_pos.y + (anim.end_pos.y - anim.start_pos.y) * ease_t,
                        );
                        let rect =
                            egui::Rect::from_center_size(pos, egui::vec2(card_width, card_height));
                        draw_mini_leg_bet_card(painter, rect, anim.color, anim.value);
                    }
                    CardFlightPhase::DisappearingUnder => {
                        // Shrink and fade at end position
                        let t = (elapsed - FLY_DURATION) / DISAPPEAR_DURATION;
                        let scale = 1.0 - t * 0.5;
                        let alpha = ((1.0 - t) * 255.0) as u8;
                        let size = egui::vec2(card_width * scale, card_height * scale);
                        let rect = egui::Rect::from_center_size(anim.end_pos, size);

                        // Draw faded card
                        let faded_color = egui::Color32::from_rgba_unmultiplied(
                            color.r(),
                            color.g(),
                            color.b(),
                            alpha,
                        );
                        painter.rect_filled(
                            rect.expand(1.0),
                            3.0,
                            egui::Color32::from_rgba_unmultiplied(60, 50, 40, alpha),
                        );
                        painter.rect_filled(
                            rect,
                            2.0,
                            egui::Color32::from_rgba_unmultiplied(245, 235, 215, alpha),
                        );
                        let top_half = egui::Rect::from_min_max(
                            rect.min,
                            egui::pos2(rect.max.x, rect.center().y),
                        );
                        painter.rect_filled(top_half.shrink(1.0), 1.0, faded_color);
                    }
                    CardFlightPhase::ReappearingInside => {
                        // Mini card fades in at player's bet area (handled by the bet display itself)
                        // Just let it fade in naturally when the bet appears in the collection
                    }
                    CardFlightPhase::Done => {
                        // Animation complete, will be cleared below
                    }
                }
            });

        // Request repaint for smooth animation
        ctx.request_repaint();

        // Clear animation when done
        if phase == CardFlightPhase::Done {
            ui_state.card_flight_animation = None;
        }
    }
}

/// Render mobile UI with restructured layout:
/// - Top panel: Player info + Camel standings
/// - Dice tents: Separate panel above HUD
/// - Bottom HUD: Actions only (no scroll)
#[allow(clippy::too_many_arguments)]
fn render_mobile_ui(
    ctx: &egui::Context,
    players: &Players,
    _pyramid: &Pyramid,
    leg_tiles: &LegBettingTiles,
    turn_state: &TurnState,
    _placed_tiles: &PlacedSpectatorTiles,
    player_leg_bets: &PlayerLegBetsStore,
    player_pyramid_tokens: &PlayerPyramidTokens,
    _race_bets: &RaceBets,
    ui_state: &mut UiState,
    camel_animations: &CamelPositionAnimations,
    _roll_action: &mut MessageWriter<RollPyramidAction>,
    leg_bet_action: &mut MessageWriter<TakeLegBetAction>,
    _race_bet_action: &mut MessageWriter<PlaceRaceBetAction>,
    _spectator_tile_action: &mut MessageWriter<PlaceSpectatorTileAction>,
    _camels: &Query<(&Camel, &BoardPosition), Without<PendingInitialMove>>,
    current_player_color: egui::Color32,
    initial_rolls: &mut Option<ResMut<crate::systems::setup::InitialSetupRolls>>,
) {
    let current = players.current_player();
    let can_act = !turn_state.action_taken
        && !current.is_ai
        && ui_state.initial_rolls_complete
        && !ui_state.show_leg_scoring;

    // === TOP PANEL: Player Info + Camel Standings ===
    let panel_response = egui::TopBottomPanel::top("mobile_info_panel")
        .frame(egui::Frame::new().fill(ctx.style().visuals.panel_fill))
        .show(ctx, |ui| {
            ui.add_space(2.0);

            // Players section with adaptive row layout
            let player_count = players.players.len();
            let available_width = ui.available_width();
            let per_row = layout::players_per_row(player_count, available_width);

            // Helper closure to draw a single player card
            // Returns the avatar rect center if this is the current player (for animation targeting)
            let draw_player_card = |ui: &mut egui::Ui,
                                    i: usize,
                                    player: &crate::components::PlayerData,
                                    card_width: f32|
             -> Option<egui::Pos2> {
                const MARGIN: f32 = 2.0;
                ui.add_space(MARGIN);
                let is_current = i == players.current_player_index;
                let player_color = PLAYER_COLORS[player.color_index % PLAYER_COLORS.len()];

                let bg_color = if is_current {
                    egui::Color32::from_rgb(40, 60, 40)
                } else {
                    egui::Color32::from_rgb(35, 35, 40)
                };

                let frame = egui::Frame::new()
                    .fill(bg_color)
                    .corner_radius(egui::CornerRadius::same(3))
                    .inner_margin(egui::Margin::same(3));

                let mut current_player_pos = None;
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Avatar
                        let avatar_size = 22.0;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(avatar_size, avatar_size),
                            egui::Sense::hover(),
                        );
                        draw_avatar(ui.painter(), rect, player.character_id, Some(player_color));

                        // Track current player position for leg bet animation
                        if is_current {
                            current_player_pos = Some(rect.center());
                        }

                        // Money
                        ui.label(egui::RichText::new(format!("${}", player.money)).size(10.0));

                        // Leg bet mini cards (overlapping)
                        if i < player_leg_bets.bets.len() && !player_leg_bets.bets[i].is_empty() {
                            let bets = &player_leg_bets.bets[i];
                            draw_overlapping_stack(
                                ui,
                                bets,
                                mobile::MINI_LEG_BET_WIDTH,
                                mobile::MINI_LEG_BET_HEIGHT,
                                mobile::MINI_LEG_BET_OVERLAP,
                                |painter, rect, bet| {
                                    draw_mini_leg_bet_indicator(
                                        painter, rect, bet.camel, bet.value,
                                    );
                                },
                            );
                        }

                        // Pyramid tokens (compact)
                        if i < player_pyramid_tokens.counts.len()
                            && player_pyramid_tokens.counts[i] > 0
                        {
                            let token_count = player_pyramid_tokens.counts[i] as usize;
                            draw_spaced_row(
                                ui,
                                token_count,
                                mobile::PYRAMID_TOKEN_SIZE,
                                mobile::PYRAMID_TOKEN_SPACING,
                                |painter, center, _| {
                                    draw_pyramid_token_icon(
                                        painter,
                                        center,
                                        mobile::PYRAMID_TOKEN_SIZE,
                                    );
                                },
                            );
                        }
                    });
                });
                ui.add_space(MARGIN);
                current_player_pos
            };

            // Calculate card width based on available space
            let available_width = ui.available_width();
            let card_width = (available_width / per_row as f32).floor();

            // Draw all players in rows
            for row_start in (0..player_count).step_by(per_row) {
                ui.add_space(2.0); // top
                ui.horizontal(|ui| {
                    let row_end = (row_start + per_row).min(player_count);
                    for i in row_start..row_end {
                        if let Some(pos) = draw_player_card(ui, i, &players.players[i], card_width)
                        {
                            ui_state.player_bet_area_pos = Some(pos);
                        }
                    }
                });
                ui.add_space(2.0);
            }

            ui.separator();
            ui.add_space(2.0);

            // Camel Standings Row (right-to-left: 1st place on right, last on left)
            let sorted_camels = get_sorted_camels(camel_animations);
            let camel_count = sorted_camels.len();

            if camel_count > 0 {
                let camel_display_width = 32.0;
                let camel_display_height = 24.0;
                let label_width = 50.0; // Space for "Current\nStandings" label
                let podium_extra_height = GOLD_PODIUM_HEIGHT + 4.0; // Extra space for podiums
                let total_camels_width = camel_display_width * sorted_camels.len() as f32;
                let camels_start_x = (ui.available_width() - total_camels_width) / 2.0;

                ui.horizontal(|ui| {
                    // "Current Standings" label on the left edge
                    let (label_rect, _) = ui.allocate_exact_size(
                        egui::vec2(label_width, camel_display_height + podium_extra_height),
                        egui::Sense::hover(),
                    );
                    ui.add_space(4.0);
                    ui.painter().text(
                        label_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Current\nStandings",
                        egui::FontId::proportional(12.0),
                        egui::Color32::from_rgb(120, 120, 120),
                    );

                    // Add space to center the camels (accounting for the label we just drew)
                    let space_after_label = (camels_start_x - label_width).max(0.0);
                    ui.add_space(space_after_label);

                    // Reverse iteration: display last place on left, 1st place on right
                    let last_place_rank = camel_count - 1;
                    for (i, (color, x_offset, podium_y)) in sorted_camels.iter().rev().enumerate() {
                        let rank = camel_count - 1 - i; // Actual rank (0 = 1st place)
                        let camel_egui_color = camel_color_to_egui(*color);
                        let border_color = egui::Color32::from_rgb(
                            (camel_egui_color.r() as f32 * 0.5) as u8,
                            (camel_egui_color.g() as f32 * 0.5) as u8,
                            (camel_egui_color.b() as f32 * 0.5) as u8,
                        );

                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(
                                camel_display_width,
                                camel_display_height + podium_extra_height,
                            ),
                            egui::Sense::hover(),
                        );

                        // Base position for camel (bottom of allocated space)
                        let base_y = rect.bottom() - camel_display_height * 0.5;

                        // Draw podium step if 1st or 2nd place
                        // Camel bottom is at base_y + camel_display_height * 0.5, podium bottom should match
                        let camel_bottom = base_y + camel_display_height * 0.5;
                        if rank == 0 {
                            // Gold podium (taller)
                            let podium_height = GOLD_PODIUM_HEIGHT;
                            let podium_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.left() + 2.0, camel_bottom - podium_height),
                                egui::vec2(camel_display_width - 4.0, podium_height),
                            );
                            // Gold colors
                            ui.painter().rect_filled(
                                podium_rect,
                                1.0,
                                egui::Color32::from_rgb(255, 215, 0),
                            );
                            ui.painter().rect_stroke(
                                podium_rect,
                                1.0,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 160, 0)),
                                egui::epaint::StrokeKind::Outside,
                            );
                        } else if rank == 1 {
                            // Silver podium (shorter)
                            let podium_height = SILVER_PODIUM_HEIGHT;
                            let podium_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.left() + 2.0, camel_bottom - podium_height),
                                egui::vec2(camel_display_width - 4.0, podium_height),
                            );
                            // Silver colors
                            ui.painter().rect_filled(
                                podium_rect,
                                1.0,
                                egui::Color32::from_rgb(192, 192, 200),
                            );
                            ui.painter().rect_stroke(
                                podium_rect,
                                1.0,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(140, 140, 150)),
                                egui::epaint::StrokeKind::Outside,
                            );
                        }

                        // Camel rect with animated podium offset (podium_y is negative when on podium)
                        let animated_rect = egui::Rect::from_center_size(
                            egui::pos2(rect.center().x + *x_offset * 0.5, base_y + *podium_y),
                            egui::vec2(camel_display_width - 4.0, camel_display_height),
                        );

                        draw_camel_silhouette(
                            ui.painter(),
                            animated_rect,
                            camel_egui_color,
                            border_color,
                        );

                        // Rank indicators with hats instead of pills
                        if rank == 0 {
                            // 1st place: Gold crown
                            draw_crown_overlay(ui.painter(), animated_rect);
                        } else if rank == 1 {
                            // 2nd place: Silver crown
                            draw_silver_crown_overlay(ui.painter(), animated_rect);
                        } else if rank == last_place_rank {
                            // Last place: Dunce cap
                            draw_dunce_cap_overlay(ui.painter(), animated_rect);
                        }
                    }
                });
            }

            ui.add_space(2.0);
        });

    if ui_state.dice_popup_timer > 0.0 && ui_state.dice_popup_delay <= 0.0 {
        egui::Area::new("dice_toast_area".into())
            .anchor(
                egui::Align2::CENTER_TOP,
                egui::vec2(0.0, panel_response.response.rect.bottom() + 4.0),
            )
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                render_dice_toast(ui, ui_state);
            });
    }

    // === BOTTOM PLAYER PANEL: Player-owned items (WITH background) ===
    egui::TopBottomPanel::bottom("mobile_player_hud").show(ctx, |ui| {
        ui.add_space(4.0);

        if can_act {
            let card_width = 45.0;
            let card_height = 55.0;
            let flip_btn_width = 26.0;
            let btn_size = card_height;
            let icon_size = 24.0;

            // Calculate total width for centering
            let total_width = btn_size + 2.0 + btn_size + 4.0 + card_width + flip_btn_width;
            let start_x = (ui.available_width() - total_width) / 2.0;

            ui.horizontal(|ui| {
                ui.add_space(start_x.max(0.0));

                // Winner bet button - square with icon inside
                let (winner_rect, winner_response) =
                    ui.allocate_exact_size(egui::vec2(btn_size, btn_size), egui::Sense::click());
                let winner_bg = if winner_response.hovered() {
                    egui::Color32::from_rgb(80, 140, 80)
                } else {
                    egui::Color32::from_rgb(60, 120, 60)
                };
                ui.painter().rect_filled(winner_rect, 4.0, winner_bg);
                ui.painter().rect_stroke(
                    winner_rect,
                    4.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 80, 40)),
                    egui::epaint::StrokeKind::Outside,
                );
                let icon_rect = egui::Rect::from_center_size(
                    winner_rect.center() + egui::vec2(0.0, -8.0),
                    egui::vec2(icon_size, icon_size),
                );
                draw_camel_with_crown(ui.painter(), icon_rect);
                ui.painter().text(
                    winner_rect.center() + egui::vec2(0.0, 16.0),
                    egui::Align2::CENTER_CENTER,
                    "Bet Winner",
                    egui::FontId::proportional(8.0),
                    egui::Color32::WHITE,
                );
                if winner_response.clicked() {
                    ui_state.show_winner_betting = true;
                }

                ui.add_space(2.0);

                // Loser bet button - square with icon inside
                let (loser_rect, loser_response) =
                    ui.allocate_exact_size(egui::vec2(btn_size, btn_size), egui::Sense::click());
                let loser_bg = if loser_response.hovered() {
                    egui::Color32::from_rgb(140, 80, 80)
                } else {
                    egui::Color32::from_rgb(120, 60, 60)
                };
                ui.painter().rect_filled(loser_rect, 4.0, loser_bg);
                ui.painter().rect_stroke(
                    loser_rect,
                    4.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 40, 40)),
                    egui::epaint::StrokeKind::Outside,
                );
                let icon_rect = egui::Rect::from_center_size(
                    loser_rect.center() + egui::vec2(0.0, -8.0),
                    egui::vec2(icon_size, icon_size),
                );
                draw_camel_with_dunce_cap(ui.painter(), icon_rect);
                ui.painter().text(
                    loser_rect.center() + egui::vec2(0.0, 16.0),
                    egui::Align2::CENTER_CENTER,
                    "Bet Loser",
                    egui::FontId::proportional(8.0),
                    egui::Color32::WHITE,
                );
                if loser_response.clicked() {
                    ui_state.show_loser_betting = true;
                }

                ui.add_space(4.0);

                // Spectator tile card
                if current.has_spectator_tile {
                    let (card_rect, card_response) = ui.allocate_exact_size(
                        egui::vec2(card_width, card_height),
                        egui::Sense::click(),
                    );

                    draw_spectator_tile_card(
                        ui.painter(),
                        card_rect,
                        current.character_id,
                        current_player_color,
                        ui_state.spectator_tile_is_oasis,
                        ui_state.spectator_tile_flip_anim,
                    );

                    if ui_state.spectator_tile_selected {
                        ui.painter().rect_stroke(
                            card_rect.expand(2.0),
                            4.0,
                            egui::Stroke::new(3.0, egui::Color32::GOLD),
                            egui::epaint::StrokeKind::Outside,
                        );
                    } else if card_response.hovered() {
                        ui.painter().rect_stroke(
                            card_rect.expand(1.0),
                            3.0,
                            egui::Stroke::new(
                                2.0,
                                egui::Color32::from_rgba_unmultiplied(255, 215, 0, 128),
                            ),
                            egui::epaint::StrokeKind::Outside,
                        );
                    }

                    if card_response.clicked() {
                        ui_state.spectator_tile_selected = !ui_state.spectator_tile_selected;
                    }

                    // Flip button
                    let (flip_rect, flip_response) = ui.allocate_exact_size(
                        egui::vec2(flip_btn_width, card_height),
                        egui::Sense::click(),
                    );
                    let flip_bg = if flip_response.hovered() {
                        egui::Color32::from_rgb(70, 70, 80)
                    } else {
                        egui::Color32::from_rgb(50, 50, 60)
                    };
                    ui.painter().rect_filled(flip_rect, 3.0, flip_bg);
                    draw_flip_icon(
                        ui.painter(),
                        flip_rect.center(),
                        flip_btn_width.min(card_height) * 0.7,
                        egui::Color32::from_rgb(200, 200, 210),
                    );
                    if flip_response.clicked() && ui_state.spectator_tile_flip_anim == 0.0 {
                        ui_state.spectator_tile_flip_anim = 0.01;
                    }
                } else {
                    let (card_rect, _) = ui.allocate_exact_size(
                        egui::vec2(card_width + flip_btn_width + 4.0, card_height),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(
                        card_rect,
                        4.0,
                        egui::Color32::from_rgba_unmultiplied(80, 80, 80, 100),
                    );
                    ui.painter().text(
                        card_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "No Tile",
                        egui::FontId::proportional(9.0),
                        egui::Color32::GRAY,
                    );
                }
            });
        } else {
            // Placeholder to maintain panel height when can_act is false
            // This prevents camera zoom changes when action_taken becomes true
            let card_height = 55.0;
            ui.allocate_space(egui::vec2(1.0, card_height));
        }

        ui.add_space(4.0);
    });

    // === SHARED ACTIONS PANEL: Roll + Leg Bets (NO background - part of central area) ===
    egui::TopBottomPanel::bottom("mobile_shared_actions")
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // No button needed - player taps pyramid to set up camels
            if ui_state.initial_rolls_complete {
                // Show leg bet cards only (pyramid is now a Bevy sprite on the game board)
                let card_width = 36.0;
                let card_height = 48.0;
                let total_width = (card_width * 5.0) + (2.0 * 4.0);
                let start_x = (ui.available_width() - total_width) / 2.0;

                ui.horizontal(|ui| {
                    ui.add_space(start_x.max(0.0));

                    // Leg bet cards (always visible, track positions)
                    for (i, color) in CamelColor::all().iter().enumerate() {
                        let color = *color;
                        if let Some(tile) = leg_tiles.top_tile(color) {
                            let sense = if can_act {
                                egui::Sense::click()
                            } else {
                                egui::Sense::hover()
                            };
                            let (rect, response) =
                                ui.allocate_exact_size(egui::vec2(card_width, card_height), sense);
                            draw_mini_leg_bet_card(ui.painter(), rect, color, tile.value);

                            // Track card position for flight animation
                            ui_state.leg_bet_card_positions[i] = Some(rect.center());

                            if can_act {
                                if response.clicked() {
                                    leg_bet_action.write(TakeLegBetAction { color });
                                }

                                if response.hovered() {
                                    ui.painter().rect_stroke(
                                        rect.expand(2.0),
                                        3.0,
                                        egui::Stroke::new(2.0, egui::Color32::GOLD),
                                        egui::epaint::StrokeKind::Outside,
                                    );
                                }
                            }
                        } else {
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(card_width, card_height),
                                egui::Sense::hover(),
                            );
                            let camel_color = camel_color_to_egui(color);
                            let faded = egui::Color32::from_rgba_unmultiplied(
                                camel_color.r(),
                                camel_color.g(),
                                camel_color.b(),
                                40,
                            );
                            ui.painter().rect_filled(rect, 3.0, faded);
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "X",
                                egui::FontId::proportional(12.0),
                                egui::Color32::GRAY,
                            );
                            ui_state.leg_bet_card_positions[i] = None;
                        }
                        ui.add_space(2.0);
                    }
                });

                // Status message overlay
                if current.is_ai && !turn_state.action_taken {
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 80.0) / 2.0);
                        ui.label(
                            egui::RichText::new("AI thinking...")
                                .italics()
                                .color(egui::Color32::GRAY),
                        );
                    });
                } else if turn_state.action_taken {
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 60.0) / 2.0);
                        ui.label(egui::RichText::new("Done").color(egui::Color32::YELLOW));
                    });
                }
            }

            ui.add_space(4.0);
        });
}

/// Get sorted camels from animation state in current race ranking order
/// Returns (color, x_offset, podium_y_offset) for each camel
fn get_sorted_camels(camel_animations: &CamelPositionAnimations) -> Vec<(CamelColor, f32, f32)> {
    // Use last_order which contains the current race ranking (1st place first)
    camel_animations
        .last_order
        .iter()
        .map(|&color| {
            let anim = camel_animations
                .positions
                .iter()
                .find(|anim| anim.color == color);
            let x_offset = anim.map(|a| a.current_y_offset).unwrap_or(0.0);
            let podium_y = anim.map(|a| a.current_podium_y).unwrap_or(0.0);
            (color, x_offset, podium_y)
        })
        .collect()
}

/// Render desktop UI with side panels
#[allow(clippy::too_many_arguments)]
fn render_desktop_ui(
    ctx: &egui::Context,
    players: &Players,
    pyramid: &Pyramid,
    leg_tiles: &LegBettingTiles,
    turn_state: &TurnState,
    _placed_tiles: &PlacedSpectatorTiles,
    player_leg_bets: &PlayerLegBetsStore,
    player_pyramid_tokens: &PlayerPyramidTokens,
    race_bets: &RaceBets,
    ui_state: &mut UiState,
    camel_animations: &CamelPositionAnimations,
    roll_action: &mut MessageWriter<RollPyramidAction>,
    leg_bet_action: &mut MessageWriter<TakeLegBetAction>,
    _race_bet_action: &mut MessageWriter<PlaceRaceBetAction>,
    _spectator_tile_action: &mut MessageWriter<PlaceSpectatorTileAction>,
    camels: &Query<(&Camel, &BoardPosition), Without<PendingInitialMove>>,
    current_player_color: egui::Color32,
    initial_rolls: &mut Option<ResMut<crate::systems::setup::InitialSetupRolls>>,
) {
    // Bottom panel - Pyramid tokens display (Dice tents are now Bevy sprites)
    egui::TopBottomPanel::bottom("dice_info").show(ctx, |ui| {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            // Show dice rolled count
            let rolled = pyramid.rolled_dice.len();
            let remaining = pyramid.dice.len();
            ui.label(egui::RichText::new(format!("Dice Rolled: {}/5", rolled)).strong());

            ui.add_space(10.0);
            ui.label(format!("({} remaining in pyramid)", remaining));

            ui.add_space(30.0);
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
                egui::vec2(
                    token_size + 10.0,
                    token_size + (tokens_remaining.saturating_sub(1)) as f32 * stack_offset + 10.0,
                ),
                egui::Sense::hover(),
            );

            // Draw remaining tokens as a stack (bottom to top)
            for i in 0..tokens_remaining {
                let y_offset = (tokens_remaining - 1 - i) as f32 * stack_offset;
                let token_center = egui::pos2(
                    stack_rect.center().x,
                    stack_rect.top() + token_size / 2.0 + y_offset + 5.0,
                );

                // Draw pyramid shape (triangle)
                let pyramid_height = token_size * 0.8;
                let pyramid_width = token_size * 0.7;

                let apex = egui::pos2(token_center.x, token_center.y - pyramid_height / 2.0);
                let base_left = egui::pos2(
                    token_center.x - pyramid_width / 2.0,
                    token_center.y + pyramid_height / 2.0,
                );
                let base_right = egui::pos2(
                    token_center.x + pyramid_width / 2.0,
                    token_center.y + pyramid_height / 2.0,
                );

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
            ui.label(
                egui::RichText::new(count_text)
                    .size(12.0)
                    .color(egui::Color32::GRAY),
            );
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

        // No button needed - player taps pyramid to set up camels
        if can_act && ui_state.initial_rolls_complete {
            ui.label(egui::RichText::new("Choose an action:").color(egui::Color32::LIGHT_GREEN));
            ui.add_space(5.0);
        }

        ui.add_enabled_ui(can_act, |ui| {
            // Roll Pyramid button - pyramid shape with flip animation
            let pyramid_size = egui::vec2(75.0, 75.0);
            let pyramid_response = draw_pyramid_button(ui, pyramid_size, ui_state.pyramid_flip_anim);
            if pyramid_response.clicked() && ui_state.pyramid_flip_anim == 0.0 {
                ui_state.pyramid_flip_anim = 0.01;  // Start flip animation
                roll_action.write(RollPyramidAction);
            }
            pyramid_response.on_hover_text("Roll a random die from the pyramid.\nYou earn $1.");

            ui.add_space(12.0);

            // Leg Betting Tiles - show as sophisticated cards with camel on top, value below
            ui.label(egui::RichText::new("Leg Bets:").size(12.0));
            ui.horizontal_wrapped(|ui| {
                for (i, color) in CamelColor::all().iter().enumerate() {
                    let color = *color;
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

                        // Track card position for flight animation
                        ui_state.leg_bet_card_positions[i] = Some(rect.center());

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
                        ui_state.leg_bet_card_positions[i] = None;

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
            if current.has_spectator_tile {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Spectator Tile:").size(12.0));

                ui.horizontal(|ui| {
                    // Draw the spectator tile card
                    let card_size = egui::vec2(50.0, 70.0);
                    let (card_rect, card_response) = ui.allocate_exact_size(card_size, egui::Sense::click());

                    // Draw the card with current flip state
                    draw_spectator_tile_card(
                        ui.painter(),
                        card_rect,
                        current.character_id,
                        current_player_color,
                        ui_state.spectator_tile_is_oasis,
                        ui_state.spectator_tile_flip_anim,
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
                        ui_state.show_spectator_tile = true;
                        ui_state.spectator_tile_space = None;
                    }
                    card_response.on_hover_text(format!(
                        "Click to place {} tile.\nEarn $1 when a camel lands on it.",
                        if ui_state.spectator_tile_is_oasis { "Oasis (+1)" } else { "Mirage (-1)" }
                    ));

                    ui.add_space(4.0);

                    // Flip button with custom icon
                    let flip_size = 24.0;
                    let (flip_rect, flip_response) = ui.allocate_exact_size(
                        egui::vec2(flip_size, flip_size),
                        egui::Sense::click()
                    );
                    let flip_bg = if flip_response.hovered() {
                        egui::Color32::from_rgb(70, 70, 80)
                    } else {
                        egui::Color32::from_rgb(50, 50, 60)
                    };
                    ui.painter().rect_filled(flip_rect, 3.0, flip_bg);
                    draw_flip_icon(ui.painter(), flip_rect.center(), flip_size * 0.75, egui::Color32::from_rgb(200, 200, 210));
                    if flip_response.clicked() && ui_state.spectator_tile_flip_anim == 0.0 {
                        // Start flip animation (will animate from 0 to 1 in update system)
                        ui_state.spectator_tile_flip_anim = 0.001; // Signal to start animation
                    }
                    flip_response.on_hover_text("Flip tile to show other side");
                });
            }

            ui.add_space(8.0);

            // Race Bet buttons - Winner and Loser as square buttons with icons inside
            ui.horizontal(|ui| {
                let btn_size = 70.0;  // Square buttons
                let icon_size = 36.0;

                // Winner bet button - square with icon inside
                let (winner_rect, winner_response) = ui.allocate_exact_size(
                    egui::vec2(btn_size, btn_size),
                    egui::Sense::click()
                );
                // Draw button background
                let winner_bg = if winner_response.hovered() {
                    egui::Color32::from_rgb(80, 140, 80)
                } else {
                    egui::Color32::from_rgb(60, 120, 60)
                };
                ui.painter().rect_filled(winner_rect, 4.0, winner_bg);
                ui.painter().rect_stroke(winner_rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 80, 40)), egui::epaint::StrokeKind::Outside);
                // Draw icon in upper portion
                let icon_rect = egui::Rect::from_center_size(
                    winner_rect.center() + egui::vec2(0.0, -10.0),
                    egui::vec2(icon_size, icon_size)
                );
                draw_camel_with_crown(ui.painter(), icon_rect);
                // Draw text below icon
                ui.painter().text(
                    winner_rect.center() + egui::vec2(0.0, 22.0),
                    egui::Align2::CENTER_CENTER,
                    "Bet Winner",
                    egui::FontId::proportional(10.0),
                    egui::Color32::WHITE
                );
                if winner_response.clicked() {
                    ui_state.show_winner_betting = true;
                }
                winner_response.on_hover_text("Bet on the race winner.\nPayouts: 1st=$8, 2nd=$5, 3rd=$3, 4th=$2, 5th+=$1\nWrong: -$1");

                ui.add_space(4.0);

                // Loser bet button - square with icon inside
                let (loser_rect, loser_response) = ui.allocate_exact_size(
                    egui::vec2(btn_size, btn_size),
                    egui::Sense::click()
                );
                // Draw button background
                let loser_bg = if loser_response.hovered() {
                    egui::Color32::from_rgb(140, 80, 80)
                } else {
                    egui::Color32::from_rgb(120, 60, 60)
                };
                ui.painter().rect_filled(loser_rect, 4.0, loser_bg);
                ui.painter().rect_stroke(loser_rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 40, 40)), egui::epaint::StrokeKind::Outside);
                // Draw icon in upper portion
                let icon_rect = egui::Rect::from_center_size(
                    loser_rect.center() + egui::vec2(0.0, -10.0),
                    egui::vec2(icon_size, icon_size)
                );
                draw_camel_with_dunce_cap(ui.painter(), icon_rect);
                // Draw text below icon
                ui.painter().text(
                    loser_rect.center() + egui::vec2(0.0, 22.0),
                    egui::Align2::CENTER_CENTER,
                    "Bet Loser",
                    egui::FontId::proportional(10.0),
                    egui::Color32::WHITE
                );
                if loser_response.clicked() {
                    ui_state.show_loser_betting = true;
                }
                loser_response.on_hover_text("Bet on the race loser.\nPayouts: 1st=$8, 2nd=$5, 3rd=$3, 4th=$2, 5th+=$1\nWrong: -$1");
            });
        });

        if turn_state.action_taken {
            ui.add_space(15.0);
            ui.label(egui::RichText::new("Action taken!").color(egui::Color32::YELLOW));
            ui.label("Advancing to next player...");
        }
    });

    // Right panel - All players and camel positions
    egui::SidePanel::right("players_list")
        .min_width(280.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Players");
                ui.separator();

                for (i, player) in players.players.iter().enumerate() {
                    let is_current = i == players.current_player_index;
                    let player_color = PLAYER_COLORS[player.color_index % PLAYER_COLORS.len()];

                    // Player header with frame for current player
                    let frame = if is_current {
                        egui::Frame::group(ui.style())
                            .stroke(egui::Stroke::new(2.0, egui::Color32::LIGHT_GREEN))
                            .inner_margin(4.0)
                    } else {
                        egui::Frame::group(ui.style()).inner_margin(4.0)
                    };

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Character avatar with colored border
                            let avatar_size = 40.0;
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(avatar_size, avatar_size),
                                egui::Sense::hover(),
                            );
                            draw_avatar(
                                ui.painter(),
                                rect,
                                player.character_id,
                                Some(player_color),
                            );

                            // Track current player's position for leg bet card animation
                            if is_current {
                                ui_state.player_bet_area_pos = Some(rect.center());
                            }

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
                                    egui::RichText::new(&player.name).size(14.0)
                                };
                                ui.label(text);

                                // Money and AI status on second line
                                let ai_tag = if player.is_ai { " (AI)" } else { "" };
                                ui.label(
                                    egui::RichText::new(format!("${}{}", player.money, ai_tag))
                                        .size(12.0),
                                );
                            });
                        });

                        // Show player's leg bets as overlapping mini cards
                        if i < player_leg_bets.bets.len() && !player_leg_bets.bets[i].is_empty() {
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.label(egui::RichText::new("Leg bets:").small());

                                let bets = &player_leg_bets.bets[i];
                                draw_overlapping_stack(
                                    ui,
                                    bets,
                                    desktop::MINI_LEG_BET_WIDTH,
                                    desktop::MINI_LEG_BET_HEIGHT,
                                    desktop::MINI_LEG_BET_OVERLAP,
                                    |painter, rect, bet| {
                                        draw_mini_leg_bet_indicator(
                                            painter, rect, bet.camel, bet.value,
                                        );
                                    },
                                );
                            });
                        }

                        // Show pyramid tokens earned this leg as individual icons
                        if i < player_pyramid_tokens.counts.len()
                            && player_pyramid_tokens.counts[i] > 0
                        {
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                let token_count = player_pyramid_tokens.counts[i] as usize;
                                draw_spaced_row(
                                    ui,
                                    token_count,
                                    desktop::PYRAMID_TOKEN_SIZE,
                                    desktop::PYRAMID_TOKEN_SPACING,
                                    |painter, center, _| {
                                        draw_pyramid_token_icon(
                                            painter,
                                            center,
                                            desktop::PYRAMID_TOKEN_SIZE,
                                        );
                                    },
                                );

                                // Show total value
                                ui.label(
                                    egui::RichText::new(format!("+${}", token_count))
                                        .small()
                                        .color(egui::Color32::GOLD),
                                );
                            });
                        }

                        // Show race bets for this player
                        let player_id = player.id;
                        let winner_bets: Vec<_> = race_bets
                            .winner_bets
                            .iter()
                            .filter(|b| b.player_id == player_id)
                            .collect();
                        let loser_bets: Vec<_> = race_bets
                            .loser_bets
                            .iter()
                            .filter(|b| b.player_id == player_id)
                            .collect();

                        if !winner_bets.is_empty() {
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.label(
                                    egui::RichText::new("Winner bets:")
                                        .small()
                                        .color(egui::Color32::LIGHT_GREEN),
                                );
                                for bet in winner_bets {
                                    let camel_color = camel_color_to_egui(bet.camel);
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(10.0, 10.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 2.0, camel_color);
                                }
                            });
                        }

                        if !loser_bets.is_empty() {
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.label(
                                    egui::RichText::new("Loser bets:")
                                        .small()
                                        .color(egui::Color32::from_rgb(255, 100, 100)),
                                );
                                for bet in loser_bets {
                                    let camel_color = camel_color_to_egui(bet.camel);
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(10.0, 10.0),
                                        egui::Sense::hover(),
                                    );
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
                    let y_offset = camel_animations
                        .positions
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
                        let (bar_rect, _) = ui.allocate_exact_size(
                            egui::vec2(bar_width, bar_height),
                            egui::Sense::hover(),
                        );

                        // Draw progress bar background
                        ui.painter().rect_filled(
                            bar_rect,
                            2.0,
                            egui::Color32::from_rgb(60, 60, 60),
                        );

                        // Draw progress bar fill with camel color
                        let camel_bar_color = camel_color_to_egui(*color);
                        let fill_width = bar_width * progress;
                        let fill_rect = egui::Rect::from_min_size(
                            bar_rect.min,
                            egui::vec2(fill_width, bar_height),
                        );
                        ui.painter().rect_filled(fill_rect, 2.0, camel_bar_color);

                        // Draw border
                        ui.painter().rect_stroke(
                            bar_rect,
                            2.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                            egui::epaint::StrokeKind::Outside,
                        );

                        // Add flexible spacer to push camel to the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Draw camel silhouette on the right with animation offset
                            let camel_egui_color = camel_color_to_egui(*color);
                            let border_color = egui::Color32::from_rgb(
                                (camel_egui_color.r() as f32 * 0.5) as u8,
                                (camel_egui_color.g() as f32 * 0.5) as u8,
                                (camel_egui_color.b() as f32 * 0.5) as u8,
                            );

                            let (rect, _) =
                                ui.allocate_exact_size(silhouette_size, egui::Sense::hover());
                            // Apply animation offset to the silhouette position
                            let animated_rect = rect.translate(egui::vec2(0.0, y_offset));
                            draw_camel_silhouette(
                                ui.painter(),
                                animated_rect,
                                camel_egui_color,
                                border_color,
                            );
                        });
                    });

                    // Add spacing between rows
                    if rank < 4 {
                        ui.add_space(4.0);
                    }
                }
            });
        });

    // Dice roll toast notification - floating below the right panel
    render_dice_toast_floating(ctx, ui_state);
}

/// Render shared popup windows (race betting, spectator tile placement, dice result)
#[allow(clippy::too_many_arguments)]
fn render_popup_windows(
    ctx: &egui::Context,
    players: &Players,
    placed_tiles: &PlacedSpectatorTiles,
    race_bets: &RaceBets,
    ui_state: &mut UiState,
    race_bet_action: &mut MessageWriter<PlaceRaceBetAction>,
    spectator_tile_action: &mut MessageWriter<PlaceSpectatorTileAction>,
    camels: &Query<(&Camel, &BoardPosition), Without<PendingInitialMove>>,
    crazy_camels: &Query<(&CrazyCamel, &BoardPosition), Without<PendingInitialMove>>,
    current_player_color: egui::Color32,
) {
    // Winner betting popup window
    if ui_state.show_winner_betting {
        egui::Window::new("Bet on Race Winner")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                let current = players.current_player();
                let player_color = PLAYER_COLORS[current.color_index % PLAYER_COLORS.len()];
                let character_id = current.character_id;

                ui.horizontal(|ui| {
                    // Left side: Payout info
                    ui.vertical(|ui| {
                        ui.set_min_width(100.0);
                        ui.heading(
                            egui::RichText::new("Payouts")
                                .size(14.0)
                                .color(egui::Color32::LIGHT_GREEN),
                        );
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Correct Bet:").size(11.0).strong());
                        ui.label(egui::RichText::new("  1st: $8").size(11.0));
                        ui.label(egui::RichText::new("  2nd: $5").size(11.0));
                        ui.label(egui::RichText::new("  3rd: $3").size(11.0));
                        ui.label(egui::RichText::new("  4th: $2").size(11.0));
                        ui.label(egui::RichText::new("  5th+: $1").size(11.0));
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Wrong: -$1")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(255, 100, 100)),
                        );
                    });

                    ui.separator();

                    // Right side: Cards
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Choose a camel you think will WIN:")
                                .size(12.0)
                                .color(egui::Color32::GRAY),
                        );
                        ui.add_space(8.0);

                        ui.horizontal_wrapped(|ui| {
                            for color in CamelColor::all() {
                                let has_card = current.available_race_cards.contains(&color);
                                let card_size = egui::vec2(70.0, 90.0);
                                let (rect, response) =
                                    ui.allocate_exact_size(card_size, egui::Sense::click());

                                if has_card {
                                    draw_race_bet_card(
                                        ui.painter(),
                                        rect,
                                        color,
                                        character_id,
                                        player_color,
                                        response.hovered(),
                                    );

                                    if response.clicked() {
                                        race_bet_action.write(PlaceRaceBetAction {
                                            color,
                                            is_winner_bet: true,
                                        });
                                        ui_state.show_winner_betting = false;
                                    }

                                    response.on_hover_text(format!("Bet on {:?} to WIN", color));
                                } else {
                                    // Determine if this card was used for winner or loser bet
                                    let bet_type = if race_bets
                                        .winner_bets
                                        .iter()
                                        .any(|b| b.player_id == current.id && b.camel == color)
                                    {
                                        PlacedBetType::Winner
                                    } else {
                                        PlacedBetType::Loser
                                    };
                                    draw_race_bet_card_unavailable(
                                        ui.painter(),
                                        rect,
                                        color,
                                        bet_type,
                                    );
                                    response
                                        .on_hover_text(format!("{:?} card already used", color));
                                }
                            }
                        });
                    });
                });

                ui.add_space(10.0);

                ui.vertical_centered(|ui| {
                    if desert_button(ui, "Cancel", &DesertButtonStyle::small()).clicked() {
                        ui_state.show_winner_betting = false;
                    }
                });
            });
    }

    // Loser betting popup window
    if ui_state.show_loser_betting {
        egui::Window::new("Bet on Race Loser")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                let current = players.current_player();
                let player_color = PLAYER_COLORS[current.color_index % PLAYER_COLORS.len()];
                let character_id = current.character_id;

                ui.horizontal(|ui| {
                    // Left side: Payout info
                    ui.vertical(|ui| {
                        ui.set_min_width(100.0);
                        ui.heading(
                            egui::RichText::new("Payouts")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(255, 100, 100)),
                        );
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Correct Bet:").size(11.0).strong());
                        ui.label(egui::RichText::new("  1st: $8").size(11.0));
                        ui.label(egui::RichText::new("  2nd: $5").size(11.0));
                        ui.label(egui::RichText::new("  3rd: $3").size(11.0));
                        ui.label(egui::RichText::new("  4th: $2").size(11.0));
                        ui.label(egui::RichText::new("  5th+: $1").size(11.0));
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Wrong: -$1")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(255, 100, 100)),
                        );
                    });

                    ui.separator();

                    // Right side: Cards
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Choose a camel you think will LOSE:")
                                .size(12.0)
                                .color(egui::Color32::GRAY),
                        );
                        ui.add_space(8.0);

                        ui.horizontal_wrapped(|ui| {
                            for color in CamelColor::all() {
                                let has_card = current.available_race_cards.contains(&color);
                                let card_size = egui::vec2(70.0, 90.0);
                                let (rect, response) =
                                    ui.allocate_exact_size(card_size, egui::Sense::click());

                                if has_card {
                                    draw_race_bet_card(
                                        ui.painter(),
                                        rect,
                                        color,
                                        character_id,
                                        player_color,
                                        response.hovered(),
                                    );

                                    if response.clicked() {
                                        race_bet_action.write(PlaceRaceBetAction {
                                            color,
                                            is_winner_bet: false,
                                        });
                                        ui_state.show_loser_betting = false;
                                    }

                                    response.on_hover_text(format!("Bet on {:?} to LOSE", color));
                                } else {
                                    // Determine if this card was used for winner or loser bet
                                    let bet_type = if race_bets
                                        .winner_bets
                                        .iter()
                                        .any(|b| b.player_id == current.id && b.camel == color)
                                    {
                                        PlacedBetType::Winner
                                    } else {
                                        PlacedBetType::Loser
                                    };
                                    draw_race_bet_card_unavailable(
                                        ui.painter(),
                                        rect,
                                        color,
                                        bet_type,
                                    );
                                    response
                                        .on_hover_text(format!("{:?} card already used", color));
                                }
                            }
                        });
                    });
                });

                ui.add_space(10.0);

                ui.vertical_centered(|ui| {
                    if desert_button(ui, "Cancel", &DesertButtonStyle::small()).clicked() {
                        ui_state.show_loser_betting = false;
                    }
                });
            });
    }

    // Spectator tile placement popup window
    if ui_state.show_spectator_tile {
        let tile_type = if ui_state.spectator_tile_is_oasis {
            "Oasis (+1)"
        } else {
            "Mirage (-1)"
        };
        egui::Window::new(format!("Place Spectator Tile ({})", tile_type))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Collect spaces with camels (including crazy camels)
                let camel_spaces: std::collections::HashSet<u8> = camels
                    .iter()
                    .map(|(_, p)| p.space_index)
                    .chain(crazy_camels.iter().map(|(_, p)| p.space_index))
                    .collect();

                // Show current tile type with mini preview
                ui.horizontal(|ui| {
                    ui.label("Placing:");
                    let (preview_rect, _) =
                        ui.allocate_exact_size(egui::vec2(36.0, 50.0), egui::Sense::hover());
                    draw_spectator_tile_card(
                        ui.painter(),
                        preview_rect,
                        players.current_player().character_id,
                        current_player_color,
                        ui_state.spectator_tile_is_oasis,
                        0.0,
                    );
                    ui.label(if ui_state.spectator_tile_is_oasis {
                        "Camels move +1 space (on top)"
                    } else {
                        "Camels move -1 space (under)"
                    });
                });

                ui.add_space(8.0);
                ui.label("Select a space (2-16):");
                ui.label(
                    egui::RichText::new(
                        "(Cannot place on space 1, spaces with camels, or other tiles)",
                    )
                    .small()
                    .color(egui::Color32::GRAY),
                );
                ui.add_space(6.0);

                // Show space selection grid
                ui.horizontal_wrapped(|ui| {
                    for space in 1..TRACK_LENGTH {
                        // Spaces 1-15 (indices 1-15), space 0 is start
                        let has_camel = camel_spaces.contains(&space);
                        let has_tile = placed_tiles.is_space_occupied(space);
                        let is_selected = ui_state.spectator_tile_space == Some(space);

                        let can_place = !has_camel && !has_tile;

                        let button_text = format!("{}", space + 1);
                        let button = if is_selected {
                            egui::Button::new(egui::RichText::new(&button_text).strong()).fill(
                                if ui_state.spectator_tile_is_oasis {
                                    egui::Color32::from_rgb(80, 160, 80)
                                } else {
                                    egui::Color32::from_rgb(200, 150, 80)
                                },
                            )
                        } else {
                            egui::Button::new(&button_text)
                        };

                        ui.add_enabled_ui(can_place, |ui| {
                            if ui.add(button).clicked() {
                                ui_state.spectator_tile_space = Some(space);
                            }
                        });
                    }
                });

                ui.add_space(10.0);

                if let Some(selected_space) = ui_state.spectator_tile_space {
                    ui.horizontal(|ui| {
                        ui.label(format!("Selected: Space {}", selected_space + 1));
                        ui.add_space(10.0);

                        // Place button with appropriate color
                        let place_btn = egui::Button::new(
                            egui::RichText::new(format!("Place {}", tile_type)).strong(),
                        )
                        .fill(if ui_state.spectator_tile_is_oasis {
                            egui::Color32::from_rgb(80, 160, 80)
                        } else {
                            egui::Color32::from_rgb(200, 150, 80)
                        });

                        if ui.add(place_btn).clicked() {
                            spectator_tile_action.write(PlaceSpectatorTileAction {
                                space_index: selected_space,
                                is_oasis: ui_state.spectator_tile_is_oasis,
                            });
                            ui_state.show_spectator_tile = false;
                            ui_state.spectator_tile_space = None;
                        }
                    });
                }

                ui.add_space(10.0);
                if desert_button(ui, "Cancel", &DesertButtonStyle::small()).clicked() {
                    ui_state.show_spectator_tile = false;
                    ui_state.spectator_tile_space = None;
                }
            });
    }
}

/// Render dice roll toast notification as a floating element that slides down from top panels
fn render_dice_toast_floating(ctx: &egui::Context, ui_state: &UiState) {
    // Only show after delay completes (waits for dice shake animation)
    if ui_state.dice_popup_timer <= 0.0 || ui_state.dice_popup_delay > 0.0 {
        return;
    }

    let Some(ref last_roll) = ui_state.last_roll else {
        return;
    };

    // Calculate animation phases
    let total_duration = match last_roll {
        LastRoll::Regular(_, _) => 2.0,
        LastRoll::Crazy(_, _) => 2.5,
    };
    let time_elapsed = (total_duration - ui_state.dice_popup_timer).max(0.0);

    // Slide-down animation during first 0.25 seconds
    let slide_duration = 0.25;
    let y_offset = if time_elapsed < slide_duration {
        let t = (time_elapsed / slide_duration).clamp(0.0, 1.0);
        // Ease-out cubic: start at -50, end at 0
        let ease_t = 1.0 - (1.0 - t).powi(3);
        -50.0 * (1.0 - ease_t)
    } else {
        0.0
    };

    // Fade out during last 0.5 seconds
    let alpha = if ui_state.dice_popup_timer < 0.5 {
        ui_state.dice_popup_timer / 0.5
    } else {
        1.0
    };
    let alpha_u8 = (alpha * 255.0) as u8;

    let (color_name, value, camel_color, is_crazy) = match last_roll {
        LastRoll::Regular(color, value) => (
            format!("{:?}", color),
            *value,
            camel_color_to_egui(*color),
            false,
        ),
        LastRoll::Crazy(color, value) => (
            format!("{:?}", color),
            *value,
            crazy_camel_color_to_egui(*color),
            true,
        ),
    };

    // Movement text
    let move_text = if is_crazy {
        format!("{} backwards!", value)
    } else {
        format!("{} spaces!", value)
    };

    // Create styled frame with camel color border
    let border_color = egui::Color32::from_rgba_unmultiplied(
        camel_color.r(),
        camel_color.g(),
        camel_color.b(),
        alpha_u8,
    );
    let bg_color = egui::Color32::from_rgba_unmultiplied(40, 40, 50, (alpha * 240.0) as u8);

    // Position the toast at top-center, sliding down from the top panel
    let toast_y = y_offset;

    egui::Area::new(egui::Id::new("dice_toast_floating"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, toast_y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let toast_frame = egui::Frame::new()
                .fill(bg_color)
                .stroke(egui::Stroke::new(2.0, border_color))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(12, 8))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 4],
                    blur: 8,
                    spread: 0,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, (alpha * 100.0) as u8),
                });

            toast_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Mini camel silhouette
                    let silhouette_size = egui::vec2(28.0, 21.0);
                    let (rect, _) = ui.allocate_exact_size(silhouette_size, egui::Sense::hover());
                    let border = egui::Color32::from_rgba_unmultiplied(
                        (camel_color.r() as f32 * 0.5) as u8,
                        (camel_color.g() as f32 * 0.5) as u8,
                        (camel_color.b() as f32 * 0.5) as u8,
                        alpha_u8,
                    );
                    let color_with_alpha = egui::Color32::from_rgba_unmultiplied(
                        camel_color.r(),
                        camel_color.g(),
                        camel_color.b(),
                        alpha_u8,
                    );
                    draw_camel_silhouette(ui.painter(), rect, color_with_alpha, border);

                    ui.add_space(4.0);

                    // Color name in camel color
                    ui.label(egui::RichText::new(&color_name).size(16.0).strong().color(
                        egui::Color32::from_rgba_unmultiplied(
                            camel_color.r(),
                            camel_color.g(),
                            camel_color.b(),
                            alpha_u8,
                        ),
                    ));

                    // "moves" text
                    ui.label(egui::RichText::new("moves").size(16.0).color(
                        egui::Color32::from_rgba_unmultiplied(200, 200, 200, alpha_u8),
                    ));

                    // Movement amount
                    ui.label(egui::RichText::new(&move_text).size(16.0).strong().color(
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8),
                    ));
                });
            });
        });

    // Request repaint for smooth animation
    ctx.request_repaint();
}

fn render_dice_toast(ui: &mut egui::Ui, ui_state: &UiState) {
    // Safety check
    let Some(ref last_roll) = ui_state.last_roll else {
        return;
    };

    // Calculate animation phases
    let total_duration = match last_roll {
        LastRoll::Regular(_, _) => 2.0,
        LastRoll::Crazy(_, _) => 2.5,
    };

    let timer = ui_state.dice_popup_timer.max(0.0);
    let time_elapsed = (total_duration - timer).max(0.0);

    // Wipe-down animation
    let wipe_duration = 0.2;
    let y_offset = if time_elapsed < wipe_duration {
        let t = (time_elapsed / wipe_duration).clamp(0.0, 1.0);
        -20.0 * (1.0 - t)
    } else {
        0.0
    };

    // Fade out
    let alpha = if timer < 0.5 { timer / 0.5 } else { 1.0 };
    let alpha_u8 = (alpha * 255.0) as u8;

    let (color_name, value, camel_color, is_crazy) = match last_roll {
        LastRoll::Regular(color, value) => (
            format!("{:?}", color),
            *value,
            camel_color_to_egui(*color),
            false,
        ),
        LastRoll::Crazy(color, value) => (
            format!("{:?}", color),
            *value,
            crazy_camel_color_to_egui(*color),
            true,
        ),
    };

    let move_text = if is_crazy {
        format!("{} backwards!", value)
    } else {
        format!("{} {}", value, if value == 1 { "space" } else { "spaces" })
    };

    let border_color = egui::Color32::from_rgba_unmultiplied(
        camel_color.r(),
        camel_color.g(),
        camel_color.b(),
        alpha_u8,
    );
    let bg_color = egui::Color32::from_rgba_unmultiplied(30, 30, 35, (alpha * 240.0) as u8);

    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(8.0 + y_offset);

        let shadow = egui::Shadow {
            offset: [2, 2],
            blur: 10,
            spread: 0,
            color: egui::Color32::from_black_alpha(96),
        };

        let toast_frame = egui::Frame::new()
            .fill(bg_color)
            .stroke(egui::Stroke::new(2.0, border_color))
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(12, 8))
            .shadow(shadow); // Use the manually created shadow

        toast_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // Mini camel silhouette
                let silhouette_size = egui::vec2(24.0, 18.0);
                let (rect, _) = ui.allocate_exact_size(silhouette_size, egui::Sense::hover());

                let fill_color = egui::Color32::from_rgba_unmultiplied(
                    camel_color.r(),
                    camel_color.g(),
                    camel_color.b(),
                    alpha_u8,
                );

                draw_camel_silhouette(ui.painter(), rect, fill_color, fill_color);

                ui.label(
                    egui::RichText::new(&color_name)
                        .size(16.0)
                        .strong()
                        .color(fill_color),
                );

                ui.label(egui::RichText::new("moves").size(14.0).color(
                    egui::Color32::from_rgba_unmultiplied(200, 200, 200, alpha_u8),
                ));

                ui.label(egui::RichText::new(&move_text).size(16.0).strong().color(
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8),
                ));
            });
        });
    });
}

/// System to update UI state when a regular die roll happens
pub fn update_ui_on_roll(
    mut events: MessageReader<PyramidRollResult>,
    mut ui_state: ResMut<UiState>,
    time: Res<Time>,
) {
    for event in events.read() {
        ui_state.last_roll = Some(LastRoll::Regular(event.color, event.value));
        ui_state.dice_popup_delay = 0.5; // Wait for shake animation to complete
        ui_state.dice_popup_timer = 2.0; // Show popup for 2 seconds after delay
                                         // Start die roll animation
        ui_state.die_roll_animation = Some(DieRollAnimation {
            die_color: Some(event.color),
            start_time: time.elapsed_secs_f64(),
        });
    }
}

/// System to update UI state when a crazy camel die roll happens
pub fn update_ui_on_crazy_roll(
    mut events: MessageReader<CrazyCamelRollResult>,
    mut ui_state: ResMut<UiState>,
    time: Res<Time>,
) {
    for event in events.read() {
        ui_state.last_roll = Some(LastRoll::Crazy(event.color, event.value));
        ui_state.dice_popup_delay = 0.5; // Wait for shake animation to complete
        ui_state.dice_popup_timer = 2.5; // Show popup for 2.5 seconds after delay
                                         // Start die roll animation (None = crazy die, gray color)
        ui_state.die_roll_animation = Some(DieRollAnimation {
            die_color: None, // Crazy die
            start_time: time.elapsed_secs_f64(),
        });
    }
}

/// System to update dice popup timer
pub fn update_dice_popup_timer(time: Res<Time>, mut ui_state: ResMut<UiState>) {
    // Count down delay first (waits for dice shake animation)
    if ui_state.dice_popup_delay > 0.0 {
        ui_state.dice_popup_delay -= time.delta_secs();
    } else if ui_state.dice_popup_timer > 0.0 {
        // Only count down popup timer after delay is complete
        ui_state.dice_popup_timer -= time.delta_secs();
    }

    // Update spectator tile flip animation
    if ui_state.spectator_tile_flip_anim > 0.0 && ui_state.spectator_tile_flip_anim < 1.0 {
        // Animate the flip over 0.3 seconds
        let flip_speed = 3.33; // 1.0 / 0.3
        ui_state.spectator_tile_flip_anim += time.delta_secs() * flip_speed;

        // When flip completes at 1.0, toggle the oasis state and reset animation
        if ui_state.spectator_tile_flip_anim >= 1.0 {
            ui_state.spectator_tile_is_oasis = !ui_state.spectator_tile_is_oasis;
            ui_state.spectator_tile_flip_anim = 0.0;
        }
    }

    // Update pyramid button flip animation
    if ui_state.pyramid_flip_anim > 0.0 {
        let flip_speed = 3.0; // ~0.33 second animation
        ui_state.pyramid_flip_anim += time.delta_secs() * flip_speed;
        if ui_state.pyramid_flip_anim >= 1.0 {
            ui_state.pyramid_flip_anim = 0.0; // Animation complete
        }
    }

    // Clear die roll animation after it completes (0.4s duration)
    if let Some(ref anim) = ui_state.die_roll_animation {
        let elapsed = time.elapsed_secs_f64() - anim.start_time;
        if elapsed > 0.4 {
            ui_state.die_roll_animation = None;
        }
    }
}

/// Row height constant for camel position animations
const CAMEL_POSITION_ROW_HEIGHT: f32 = 46.0; // row height including spacing
const CAMEL_POSITION_ANIMATION_SPEED: f32 = 8.0; // How fast positions animate
const PODIUM_ANIMATION_SPEED: f32 = 6.0; // How fast podium hop animates

/// Podium heights for standings display
pub const GOLD_PODIUM_HEIGHT: f32 = 10.0; // 1st place podium (twice silver)
pub const SILVER_PODIUM_HEIGHT: f32 = 5.0; // 2nd place podium

/// System to update camel position animations
pub fn update_camel_position_animations(
    time: Res<Time>,
    mut animations: ResMut<CamelPositionAnimations>,
    camels: Query<(&Camel, &BoardPosition), Without<PendingInitialMove>>,
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

            // Calculate target podium height based on rank
            let target_podium = if new_rank == 0 {
                -GOLD_PODIUM_HEIGHT // 1st place - gold podium (negative = up)
            } else if new_rank == 1 {
                -SILVER_PODIUM_HEIGHT // 2nd place - silver podium
            } else {
                0.0 // No podium for others
            };

            // Find or create animation entry for this color
            let anim = animations.positions.iter_mut().find(|a| a.color == color);

            if let Some(anim) = anim {
                if let Some(old_rank) = old_rank {
                    // Set current offset to where the camel appears to be coming from
                    let rank_difference = new_rank as i32 - old_rank as i32;
                    anim.current_y_offset = -rank_difference as f32 * CAMEL_POSITION_ROW_HEIGHT;
                }
                anim.target_y_offset = 0.0;
                anim.target_podium_y = target_podium;
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
                    current_podium_y: target_podium, // Start at target for new camels
                    target_podium_y: target_podium,
                });
            }
        }

        animations.last_order = current_order.clone();
    }

    // Update target podium heights based on current order (even if order hasn't changed)
    // This ensures podiums are correct after initialization
    for (rank, &color) in current_order.iter().enumerate() {
        let target_podium = if rank == 0 {
            -GOLD_PODIUM_HEIGHT
        } else if rank == 1 {
            -SILVER_PODIUM_HEIGHT
        } else {
            0.0
        };

        if let Some(anim) = animations.positions.iter_mut().find(|a| a.color == color) {
            anim.target_podium_y = target_podium;
        }
    }

    // Animate offsets towards targets
    let dt = time.delta_secs();
    for anim in &mut animations.positions {
        // Animate horizontal slide
        if (anim.current_y_offset - anim.target_y_offset).abs() > 0.1 {
            let direction = if anim.current_y_offset < anim.target_y_offset {
                1.0
            } else {
                -1.0
            };
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

        // Animate podium hop (vertical)
        if (anim.current_podium_y - anim.target_podium_y).abs() > 0.1 {
            let direction = if anim.current_podium_y < anim.target_podium_y {
                1.0
            } else {
                -1.0
            };
            let step = PODIUM_ANIMATION_SPEED * dt * GOLD_PODIUM_HEIGHT;
            anim.current_podium_y += direction * step;

            // Clamp to target if we overshoot
            if direction > 0.0 && anim.current_podium_y > anim.target_podium_y {
                anim.current_podium_y = anim.target_podium_y;
            } else if direction < 0.0 && anim.current_podium_y < anim.target_podium_y {
                anim.current_podium_y = anim.target_podium_y;
            }
        } else {
            anim.current_podium_y = anim.target_podium_y;
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
    mut placed_tiles: Option<ResMut<PlacedSpectatorTiles>>,
    camels: Query<(&Camel, &BoardPosition), Without<PendingInitialMove>>,
    spectator_tile_entities: Query<Entity, With<SpectatorTile>>,
    dice_sprite_entities: Query<Entity, With<crate::systems::animation::DiceSprite>>,
    mut commands: Commands,
) {
    if !ui_state.show_leg_scoring {
        return;
    }

    let Some(ref mut players) = players else {
        return;
    };
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

            score_changes.push((
                player.name.clone(),
                leg_bet_total,
                bet_details,
                pyramid_tokens,
            ));
        }
    }

    // Get current standings for display (INCLUDING leg earnings from this leg)
    let mut sorted_players: Vec<_> = players
        .players
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            // Calculate total leg earnings for this player
            let leg_earnings = score_changes
                .get(idx)
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
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(40.0, 30.0),
                                    egui::Sense::hover(),
                                );
                                draw_camel_silhouette(ui.painter(), rect, color, border_color);
                                ui.label(
                                    egui::RichText::new(format!("{:?}", first))
                                        .size(16.0)
                                        .strong(),
                                );
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
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(40.0, 30.0),
                                    egui::Sense::hover(),
                                );
                                draw_camel_silhouette(ui.painter(), rect, color, border_color);
                                ui.label(
                                    egui::RichText::new(format!("{:?}", second))
                                        .size(16.0)
                                        .strong(),
                                );
                            });
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Show score changes for each player
                        // Only show heading on desktop to save vertical space on mobile
                        if ui_state.use_side_panels {
                            ui.heading(egui::RichText::new("Leg Earnings").size(20.0));
                            ui.add_space(10.0);
                        }

                        let is_mobile = !ui_state.use_side_panels;

                        for (name, leg_bet_total, details, pyramid_tokens) in &score_changes {
                            let has_bets = !details.is_empty();
                            let has_tokens = *pyramid_tokens > 0;

                            if has_bets || has_tokens {
                                if is_mobile {
                                    // MOBILE LAYOUT - responsive wrapping
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(egui::RichText::new(name).strong().size(13.0));
                                        ui.label(":");

                                        // Cards WITH individual earnings
                                        let card_size = egui::vec2(22.0, 30.0);
                                        for (camel, value, change) in details {
                                            let (rect, _) = ui.allocate_exact_size(
                                                card_size,
                                                egui::Sense::hover(),
                                            );
                                            draw_mini_leg_bet_card(
                                                ui.painter(),
                                                rect,
                                                *camel,
                                                *value,
                                            );

                                            let change_text = if *change > 0 {
                                                format!("+${}", change)
                                            } else {
                                                "-$1".to_string()
                                            };
                                            let change_color = if *change > 0 {
                                                egui::Color32::LIGHT_GREEN
                                            } else {
                                                egui::Color32::from_rgb(255, 100, 100)
                                            };
                                            ui.label(
                                                egui::RichText::new(&change_text)
                                                    .size(11.0)
                                                    .color(change_color),
                                            );
                                            ui.add_space(2.0);
                                        }

                                        // Compact pyramid tokens (smaller, overlapping) with earnings
                                        if *pyramid_tokens > 0 {
                                            ui.add_space(4.0);
                                            let token_size = 16.0;
                                            let token_spacing = 12.0;
                                            let total_width = token_size
                                                + (pyramid_tokens.saturating_sub(1) as f32
                                                    * token_spacing);
                                            let (tokens_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(total_width, token_size),
                                                egui::Sense::hover(),
                                            );
                                            for t in 0..*pyramid_tokens {
                                                let token_center = egui::pos2(
                                                    tokens_rect.left()
                                                        + token_size / 2.0
                                                        + (t as f32 * token_spacing),
                                                    tokens_rect.center().y,
                                                );
                                                draw_pyramid_token_icon(
                                                    ui.painter(),
                                                    token_center,
                                                    token_size,
                                                );
                                            }
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "+${}",
                                                    pyramid_tokens
                                                ))
                                                .size(11.0)
                                                .color(egui::Color32::GOLD),
                                            );
                                        }

                                        // Total - wraps only if needed
                                        let total = *leg_bet_total + (*pyramid_tokens as i32);
                                        if total != 0 {
                                            ui.add_space(8.0);
                                            let (text, color) = if total > 0 {
                                                (
                                                    format!("= +${}", total),
                                                    egui::Color32::LIGHT_GREEN,
                                                )
                                            } else {
                                                (
                                                    format!("= -${}", total.abs()),
                                                    egui::Color32::from_rgb(255, 100, 100),
                                                )
                                            };
                                            ui.label(
                                                egui::RichText::new(text)
                                                    .strong()
                                                    .size(13.0)
                                                    .color(color),
                                            );
                                        }
                                    });
                                    ui.add_space(4.0);
                                } else {
                                    // DESKTOP LAYOUT - single row with all details
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(name).strong().size(14.0));
                                        ui.label(":");
                                        ui.add_space(8.0);

                                        // Show leg bet cards with results
                                        for (camel, value, change) in details {
                                            // Draw mini leg bet card
                                            let card_size = egui::vec2(28.0, 38.0);
                                            let (rect, _) = ui.allocate_exact_size(
                                                card_size,
                                                egui::Sense::hover(),
                                            );
                                            draw_mini_leg_bet_card(
                                                ui.painter(),
                                                rect,
                                                *camel,
                                                *value,
                                            );

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
                                            ui.label(
                                                egui::RichText::new(&change_text)
                                                    .size(12.0)
                                                    .color(change_color),
                                            );
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
                                            let total_width = token_size
                                                + (pyramid_tokens.saturating_sub(1) as f32
                                                    * token_spacing);
                                            let (tokens_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(total_width, token_size),
                                                egui::Sense::hover(),
                                            );

                                            // Draw each token
                                            for t in 0..*pyramid_tokens {
                                                let token_center = egui::pos2(
                                                    tokens_rect.left()
                                                        + token_size / 2.0
                                                        + (t as f32 * token_spacing),
                                                    tokens_rect.center().y,
                                                );
                                                draw_pyramid_token_icon(
                                                    ui.painter(),
                                                    token_center,
                                                    token_size,
                                                );
                                            }

                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "+${}",
                                                    pyramid_tokens
                                                ))
                                                .size(12.0)
                                                .color(egui::Color32::GOLD),
                                            );
                                        }

                                        // Show total for this leg
                                        let total_leg_earnings =
                                            *leg_bet_total + (*pyramid_tokens as i32);
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
                                            ui.label(
                                                egui::RichText::new(&total_text)
                                                    .strong()
                                                    .size(14.0)
                                                    .color(total_color),
                                            );
                                        }
                                    });
                                    ui.add_space(4.0);
                                }
                            }
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Current standings
                        // Only show heading on desktop to save vertical space on mobile
                        if ui_state.use_side_panels {
                            ui.heading(egui::RichText::new("Current Standings").size(20.0));
                        }

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

                        if desert_button(ui, "Start Next Leg", &DesertButtonStyle::medium())
                            .clicked()
                        {
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

        // Clear placed spectator tiles and return them to players
        if let Some(ref mut placed_tiles) = placed_tiles {
            placed_tiles.clear();
        }

        // Return spectator tiles to all players
        for player in players.players.iter_mut() {
            player.has_spectator_tile = true;
        }

        // Despawn visual spectator tile entities
        for entity in spectator_tile_entities.iter() {
            commands.entity(entity).despawn();
        }

        // Despawn dice sprites in tents from previous leg
        for entity in dice_sprite_entities.iter() {
            commands.entity(entity).despawn();
        }

        ui_state.show_leg_scoring = false;
    }
}
