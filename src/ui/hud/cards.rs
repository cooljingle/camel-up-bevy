//! Card and betting tile drawing utilities.
//! Provides functions to draw leg bet cards, race bet cards, spectator tiles, and pyramid tokens.

use bevy_egui::egui;
use crate::components::CamelColor;
use crate::ui::characters::{draw_avatar, CharacterId};
use crate::ui::theme::camel_color_to_egui;
use super::camel_display::{draw_camel_silhouette, draw_crown_overlay, draw_dunce_cap_overlay};

/// Indicates what type of race bet was placed for displaying on unavailable cards
#[derive(Clone, Copy)]
pub enum PlacedBetType {
    Winner,
    Loser,
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

    // Bottom half - cream background with gold coin
    painter.rect_filled(bottom_half.shrink(1.0), 1.0, egui::Color32::from_rgb(250, 245, 230));

    // Draw gold coin with value
    let coin_center = bottom_half.center();
    let coin_radius = (bottom_half.height() * 0.38).min(bottom_half.width() * 0.38);

    // Gold colors (matching pyramid token)
    let gold_light = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);
    let gold_dark = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);
    let gold_outline = egui::Color32::from_rgb(0x6B, 0x4A, 0x1A);

    // Outer shadow/depth
    painter.circle_filled(coin_center + egui::vec2(1.0, 1.0), coin_radius, gold_outline);
    // Main coin body
    painter.circle_filled(coin_center, coin_radius, gold_light);
    // Inner shadow ring for depth
    painter.circle_stroke(coin_center, coin_radius * 0.85, egui::Stroke::new(1.0, gold_dark));
    // Outer edge
    painter.circle_stroke(coin_center, coin_radius, egui::Stroke::new(1.5, gold_outline));

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
pub fn draw_mini_leg_bet_indicator(painter: &egui::Painter, rect: egui::Rect, camel_color: CamelColor, value: u8) {
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
        (egui::Color32::from_rgb(0xE4, 0xB8, 0x5B), egui::Color32::from_rgb(0xB0, 0x8A, 0x40))
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
    draw_spectator_tile_content(&clipped_painter, rect, character_id, player_color, showing_oasis);
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
    painter.rect_filled(rect.expand(2.0), 5.0, egui::Color32::from_rgb(40, 30, 20));

    // Card background
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(245, 235, 215));

    // Split into top (avatar) and bottom (value)
    let top_half = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.max.x, rect.center().y + 4.0),
    );
    let bottom_half = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, rect.center().y + 4.0),
        rect.max,
    );

    // Top half - cream background with avatar
    painter.rect_filled(top_half.shrink(2.0), 2.0, egui::Color32::from_rgb(250, 245, 230));

    // Draw player avatar in top portion at full size
    let avatar_size = (top_half.height() - 8.0).min(top_half.width() - 8.0);
    let avatar_rect = egui::Rect::from_center_size(
        top_half.center(),
        egui::vec2(avatar_size, avatar_size),
    );
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
pub fn draw_flip_icon(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    let r = size * 0.38;  // Radius of the circular path
    let stroke = egui::Stroke::new(size * 0.12, color);
    let arrow_size = size * 0.15;

    // Draw using line segments to approximate arcs
    // Top arc (right half of circle, pointing right)
    let segments = 8;
    for i in 0..segments {
        let angle1 = std::f32::consts::PI * 0.15 + (i as f32 / segments as f32) * std::f32::consts::PI * 0.7;
        let angle2 = std::f32::consts::PI * 0.15 + ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 0.7;
        let p1 = center + egui::vec2(r * angle1.cos(), -r * angle1.sin());
        let p2 = center + egui::vec2(r * angle2.cos(), -r * angle2.sin());
        painter.line_segment([p1, p2], stroke);
    }

    // Bottom arc (left half of circle, pointing left)
    for i in 0..segments {
        let angle1 = std::f32::consts::PI * 1.15 + (i as f32 / segments as f32) * std::f32::consts::PI * 0.7;
        let angle2 = std::f32::consts::PI * 1.15 + ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 0.7;
        let p1 = center + egui::vec2(r * angle1.cos(), -r * angle1.sin());
        let p2 = center + egui::vec2(r * angle2.cos(), -r * angle2.sin());
        painter.line_segment([p1, p2], stroke);
    }

    // Arrow head on top arc (pointing right/down)
    let top_arrow_pos = center + egui::vec2(r * 0.85, -r * 0.5);
    painter.line_segment(
        [top_arrow_pos, top_arrow_pos + egui::vec2(-arrow_size, -arrow_size * 0.5)],
        stroke,
    );
    painter.line_segment(
        [top_arrow_pos, top_arrow_pos + egui::vec2(-arrow_size * 0.3, arrow_size)],
        stroke,
    );

    // Arrow head on bottom arc (pointing left/up)
    let bottom_arrow_pos = center + egui::vec2(-r * 0.85, r * 0.5);
    painter.line_segment(
        [bottom_arrow_pos, bottom_arrow_pos + egui::vec2(arrow_size, arrow_size * 0.5)],
        stroke,
    );
    painter.line_segment(
        [bottom_arrow_pos, bottom_arrow_pos + egui::vec2(arrow_size * 0.3, -arrow_size)],
        stroke,
    );

    // Small center circle (eye/viewer symbol)
    painter.circle_filled(center, size * 0.12, color);
}

/// Helper function to draw a race bet card (player avatar on camel color background)
pub fn draw_race_bet_card(
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
/// Shows a camel with crown (winner bet) or dunce cap (loser bet) instead of an X
pub fn draw_race_bet_card_unavailable(painter: &egui::Painter, rect: egui::Rect, camel_color: CamelColor, placed_bet: PlacedBetType) {
    let color = camel_color_to_egui(camel_color);
    let faded_color = egui::Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        100,  // Slightly more visible than before since we're showing content
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
        rect.center() + egui::vec2(0.0, -5.0),  // Shift up slightly to make room for label
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
