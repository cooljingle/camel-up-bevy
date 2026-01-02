//! Camel drawing utilities for UI elements.
//! Provides functions to draw camel silhouettes, crowns, and dunce caps.

use bevy_egui::egui;

/// Helper function to draw a small camel silhouette for UI elements
/// Draws a stylized side-view camel using 4 layers (shadow, border, main, highlight)
/// to match the polished look of the board camels
pub fn draw_camel_silhouette(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32, border_color: egui::Color32) {
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
        body_center + egui::vec2(-5.0 * scale, 7.0 * scale),   // Back left
        body_center + egui::vec2(-2.0 * scale, 7.0 * scale),   // Back right
        body_center + egui::vec2(4.0 * scale, 7.0 * scale),    // Front left
        body_center + egui::vec2(7.0 * scale, 7.0 * scale),    // Front right
    ];

    // === Layer 1: SHADOW ===
    let shadow_offset = egui::vec2(1.5 * scale, 1.5 * scale);
    let shadow_color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 76); // ~0.3 alpha

    painter.rect_filled(body_rect.translate(shadow_offset), 1.0 * scale, shadow_color);
    painter.rect_filled(hump_rect.translate(shadow_offset), 1.0 * scale, shadow_color);
    painter.rect_filled(neck_rect.translate(shadow_offset), 0.5 * scale, shadow_color);
    painter.rect_filled(head_rect.translate(shadow_offset), 1.0 * scale, shadow_color);
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
        painter.rect_filled(leg_rect.expand(border_expand * 0.5), 0.5 * scale, border_color);
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
        egui::vec2((hump_width - 2.0 * scale).max(2.0), 2.0 * scale)
    );
    painter.rect_filled(hump_highlight_rect, 0.5 * scale, highlight_color);

    // Highlight strip on head
    let head_highlight_rect = egui::Rect::from_center_size(
        head_center + egui::vec2(0.0, -1.5 * scale),
        egui::vec2((head_width - 2.0 * scale).max(2.0), 1.5 * scale)
    );
    painter.rect_filled(head_highlight_rect, 0.5 * scale, highlight_color);

    // === Eye ===
    let eye_pos = head_center + egui::vec2(1.5 * scale, -0.5 * scale);
    painter.circle_filled(eye_pos, 1.0 * scale, egui::Color32::from_rgb(30, 30, 30));
}

/// Draws a grey camel with a gold crown on its head (winner icon)
pub fn draw_camel_with_crown(painter: &egui::Painter, rect: egui::Rect) {
    let camel_color = egui::Color32::from_rgb(140, 140, 140);  // Grey camel
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
        egui::vec2(crown_width, crown_height * 0.5)
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
            egui::pos2(point_center.x, point_center.y - point_height),  // Top
            egui::pos2(point_center.x - point_width / 2.0, point_center.y),  // Bottom left
            egui::pos2(point_center.x + point_width / 2.0, point_center.y),  // Bottom right
        ];
        painter.add(egui::Shape::convex_polygon(points, gold, egui::Stroke::new(0.5 * scale, gold_dark)));
    }

    // Small gems on crown points
    let gem_colors = [
        egui::Color32::from_rgb(220, 50, 50),   // Red
        egui::Color32::from_rgb(50, 100, 220),  // Blue
        egui::Color32::from_rgb(220, 50, 50),   // Red
    ];
    for i in 0..3 {
        let x_offset = (i as f32 - 1.0) * 2.5 * scale;
        let gem_pos = egui::pos2(crown_center.x + x_offset, point_y - 2.0 * scale);
        painter.circle_filled(gem_pos, 1.0 * scale, gem_colors[i]);
    }
}

/// Draws a grey camel with a dunce cap on its head (loser icon)
pub fn draw_camel_with_dunce_cap(painter: &egui::Painter, rect: egui::Rect) {
    let camel_color = egui::Color32::from_rgb(140, 140, 140);  // Grey camel
    let border_color = egui::Color32::from_rgb(80, 80, 80);

    // Draw the camel first
    draw_camel_silhouette(painter, rect, camel_color, border_color);

    // Draw dunce cap overlay
    draw_dunce_cap_overlay(painter, rect);
}

/// Draws just a gold crown overlay on top of a camel silhouette (1st place)
/// The rect should be the same rect used for draw_camel_silhouette
pub fn draw_crown_overlay(painter: &egui::Painter, rect: egui::Rect) {
    draw_crown_overlay_with_color(painter, rect, CrownColor::Gold);
}

/// Draws a silver crown overlay on top of a camel silhouette (2nd place)
/// The rect should be the same rect used for draw_camel_silhouette
pub fn draw_silver_crown_overlay(painter: &egui::Painter, rect: egui::Rect) {
    draw_crown_overlay_with_color(painter, rect, CrownColor::Silver);
}

/// Crown color variants
#[derive(Clone, Copy)]
pub enum CrownColor {
    Gold,
    Silver,
}

/// Draws a crown overlay with specified color
fn draw_crown_overlay_with_color(painter: &egui::Painter, rect: egui::Rect, crown_color: CrownColor) {
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

    // Crown colors based on type
    let (main_color, dark_color, gem_colors) = match crown_color {
        CrownColor::Gold => (
            egui::Color32::from_rgb(255, 215, 0),   // Gold
            egui::Color32::from_rgb(200, 160, 0),   // Dark gold
            [
                egui::Color32::from_rgb(220, 50, 50),   // Red
                egui::Color32::from_rgb(50, 100, 220),  // Blue
                egui::Color32::from_rgb(220, 50, 50),   // Red
            ],
        ),
        CrownColor::Silver => (
            egui::Color32::from_rgb(200, 200, 210),  // Silver
            egui::Color32::from_rgb(140, 140, 150),  // Dark silver
            [
                egui::Color32::from_rgb(100, 180, 220),  // Light blue
                egui::Color32::from_rgb(180, 180, 200),  // Pearl
                egui::Color32::from_rgb(100, 180, 220),  // Light blue
            ],
        ),
    };

    // Crown base (rectangle)
    let base_rect = egui::Rect::from_center_size(
        crown_center + egui::vec2(0.0, 1.5 * scale),
        egui::vec2(crown_width, crown_height * 0.5)
    );
    painter.rect_filled(base_rect, 1.0 * scale, main_color);

    // Crown points (3 triangles)
    let point_height = 4.0 * scale;
    let point_width = 2.5 * scale;
    let point_y = crown_center.y - 1.0 * scale;

    for i in 0..3 {
        let x_offset = (i as f32 - 1.0) * 2.5 * scale;
        let point_center = egui::pos2(crown_center.x + x_offset, point_y);

        // Triangle for crown point
        let points = vec![
            egui::pos2(point_center.x, point_center.y - point_height),  // Top
            egui::pos2(point_center.x - point_width / 2.0, point_center.y),  // Bottom left
            egui::pos2(point_center.x + point_width / 2.0, point_center.y),  // Bottom right
        ];
        painter.add(egui::Shape::convex_polygon(points, main_color, egui::Stroke::new(0.5 * scale, dark_color)));
    }

    // Small gems on crown points
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
    let cap_color = egui::Color32::from_rgb(100, 100, 110);  // Muted grey-blue
    let cap_outline = egui::Color32::from_rgb(60, 60, 70);

    // Dunce cap triangle
    let points = vec![
        egui::pos2(cap_base.x, cap_base.y - cap_height),  // Top point
        egui::pos2(cap_base.x - cap_width / 2.0, cap_base.y),  // Bottom left
        egui::pos2(cap_base.x + cap_width / 2.0, cap_base.y),  // Bottom right
    ];
    painter.add(egui::Shape::convex_polygon(points, cap_color, egui::Stroke::new(0.8 * scale, cap_outline)));

    // Chin strap
    let strap_color = egui::Color32::from_rgb(70, 70, 70);
    let chin_left = head_center + egui::vec2(-3.0 * scale, 2.0 * scale);
    let chin_right = head_center + egui::vec2(3.0 * scale, 2.0 * scale);
    painter.line_segment([chin_left, chin_right], egui::Stroke::new(0.5 * scale, strap_color));
}
