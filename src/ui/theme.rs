//! Desert/Pyramid themed UI widgets
//! Custom-drawn widgets that replace standard egui styling

use bevy_egui::{egui, EguiContexts};

// ============================================================================
// Color Palette - Desert Theme
// ============================================================================

/// Sand - warm tan for backgrounds
#[allow(dead_code)]
pub const SAND: egui::Color32 = egui::Color32::from_rgb(0xED, 0xC9, 0x9A);

/// Papyrus/Parchment - cream color for cards
pub const PAPYRUS: egui::Color32 = egui::Color32::from_rgb(0xF5, 0xEB, 0xD7);

/// Papyrus dark - aged edges
pub const PAPYRUS_DARK: egui::Color32 = egui::Color32::from_rgb(0xD9, 0xC9, 0xA5);

/// Pyramid gold light - sunlit stone, highlights
pub const GOLD_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);

/// Pyramid gold dark - shadowed stone
pub const GOLD_DARK: egui::Color32 = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);

/// Pyramid outline - dark brown for borders and text
pub const GOLD_OUTLINE: egui::Color32 = egui::Color32::from_rgb(0x6B, 0x4A, 0x1A);

/// Stone base - neutral gray-brown for inactive elements
pub const STONE: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x7B, 0x6A);

/// Stone light - highlight for bevels
pub const STONE_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xA8, 0x98, 0x85);

/// Stone dark - shadow for bevels
pub const STONE_DARK: egui::Color32 = egui::Color32::from_rgb(0x5A, 0x4D, 0x40);

/// Sky blue - accent color
#[allow(dead_code)]
pub const SKY_BLUE: egui::Color32 = egui::Color32::from_rgb(0x87, 0xCE, 0xEB);

/// Terracotta - warm accent
#[allow(dead_code)]
pub const TERRACOTTA: egui::Color32 = egui::Color32::from_rgb(0xC4, 0x5C, 0x3B);

// ============================================================================
// Desert Button - Stone tablet style
// ============================================================================

/// Configuration for desert button appearance
pub struct DesertButtonStyle {
    pub min_size: egui::Vec2,
    pub corner_radius: f32,
    pub font_size: f32,
}

impl Default for DesertButtonStyle {
    fn default() -> Self {
        Self {
            min_size: egui::vec2(120.0, 40.0),
            corner_radius: 6.0,
            font_size: 16.0,
        }
    }
}

impl DesertButtonStyle {
    pub fn large() -> Self {
        Self {
            min_size: egui::vec2(200.0, 50.0),
            corner_radius: 8.0,
            font_size: 24.0,
        }
    }

    pub fn medium() -> Self {
        Self {
            min_size: egui::vec2(140.0, 44.0),
            corner_radius: 6.0,
            font_size: 18.0,
        }
    }

    pub fn small() -> Self {
        Self {
            min_size: egui::vec2(80.0, 32.0),
            corner_radius: 4.0,
            font_size: 14.0,
        }
    }

    pub fn compact() -> Self {
        Self {
            min_size: egui::vec2(36.0, 36.0),
            corner_radius: 4.0,
            font_size: 18.0,
        }
    }
}

/// Draw a desert-themed stone tablet button
/// Returns the response for click handling
pub fn desert_button(ui: &mut egui::Ui, text: &str, style: &DesertButtonStyle) -> egui::Response {
    desert_button_impl(ui, text, style, true)
}

/// Draw a desert-themed button that can be disabled
pub fn desert_button_enabled(
    ui: &mut egui::Ui,
    text: &str,
    style: &DesertButtonStyle,
    enabled: bool,
) -> egui::Response {
    desert_button_impl(ui, text, style, enabled)
}

fn desert_button_impl(
    ui: &mut egui::Ui,
    text: &str,
    style: &DesertButtonStyle,
    enabled: bool,
) -> egui::Response {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };

    let (rect, response) = ui.allocate_exact_size(style.min_size, sense);

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Determine visual state
        let (base_color, bevel_invert, glow) = if !enabled {
            // Disabled - grayed out
            (
                egui::Color32::from_rgb(0x70, 0x68, 0x60),
                false,
                false,
            )
        } else if response.is_pointer_button_down_on() {
            // Pressed - inverted bevel, darker
            (STONE_DARK, true, false)
        } else if response.hovered() {
            // Hovered - golden glow
            (STONE, false, true)
        } else {
            // Normal
            (STONE, false, false)
        };

        // Draw drop shadow
        let shadow_offset = if response.is_pointer_button_down_on() {
            egui::vec2(1.0, 1.0)
        } else {
            egui::vec2(3.0, 3.0)
        };
        let shadow_rect = rect.translate(shadow_offset);
        painter.rect_filled(
            shadow_rect,
            style.corner_radius,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60),
        );

        // Draw main button body
        painter.rect_filled(rect, style.corner_radius, base_color);

        // Draw bevel edges for 3D effect
        let bevel_width = 2.0;
        let (highlight, shadow) = if bevel_invert {
            (
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            )
        } else {
            (
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60),
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80),
            )
        };

        // Top edge highlight
        painter.line_segment(
            [
                rect.left_top() + egui::vec2(style.corner_radius, 0.0),
                rect.right_top() - egui::vec2(style.corner_radius, 0.0),
            ],
            egui::Stroke::new(bevel_width, highlight),
        );
        // Left edge highlight
        painter.line_segment(
            [
                rect.left_top() + egui::vec2(0.0, style.corner_radius),
                rect.left_bottom() - egui::vec2(0.0, style.corner_radius),
            ],
            egui::Stroke::new(bevel_width, highlight),
        );
        // Bottom edge shadow
        painter.line_segment(
            [
                rect.left_bottom() + egui::vec2(style.corner_radius, 0.0),
                rect.right_bottom() - egui::vec2(style.corner_radius, 0.0),
            ],
            egui::Stroke::new(bevel_width, shadow),
        );
        // Right edge shadow
        painter.line_segment(
            [
                rect.right_top() + egui::vec2(0.0, style.corner_radius),
                rect.right_bottom() - egui::vec2(0.0, style.corner_radius),
            ],
            egui::Stroke::new(bevel_width, shadow),
        );

        // Draw golden glow on hover
        if glow {
            painter.rect_stroke(
                rect.shrink(1.0),
                style.corner_radius - 1.0,
                egui::Stroke::new(2.0, GOLD_LIGHT),
                egui::epaint::StrokeKind::Outside,
            );
        }

        // Draw outer border
        let border_color = if enabled { GOLD_OUTLINE } else { STONE_DARK };
        painter.rect_stroke(
            rect,
            style.corner_radius,
            egui::Stroke::new(1.5, border_color),
            egui::epaint::StrokeKind::Outside,
        );

        // Draw decorative corner accents (small triangles)
        let accent_size = 6.0;
        let accent_color = if glow {
            GOLD_LIGHT
        } else if enabled {
            GOLD_DARK
        } else {
            STONE_DARK
        };

        // Top-left corner accent
        draw_corner_accent(painter, rect.left_top(), accent_size, accent_color, false, false);
        // Top-right corner accent
        draw_corner_accent(painter, rect.right_top(), accent_size, accent_color, true, false);
        // Bottom-left corner accent
        draw_corner_accent(painter, rect.left_bottom(), accent_size, accent_color, false, true);
        // Bottom-right corner accent
        draw_corner_accent(painter, rect.right_bottom(), accent_size, accent_color, true, true);

        // Draw text
        let text_color = if enabled {
            PAPYRUS
        } else {
            egui::Color32::from_rgb(0x90, 0x88, 0x80)
        };

        // Text shadow for depth
        painter.text(
            rect.center() + egui::vec2(1.0, 1.0),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(style.font_size),
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100),
        );

        // Main text
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(style.font_size),
            text_color,
        );
    }

    response
}

/// Draw a small decorative corner accent
fn draw_corner_accent(
    painter: &egui::Painter,
    corner: egui::Pos2,
    size: f32,
    color: egui::Color32,
    flip_x: bool,
    flip_y: bool,
) {
    let dx = if flip_x { -size } else { size };
    let dy = if flip_y { -size } else { size };

    // Small L-shaped accent
    painter.line_segment(
        [corner, corner + egui::vec2(dx, 0.0)],
        egui::Stroke::new(2.0, color),
    );
    painter.line_segment(
        [corner, corner + egui::vec2(0.0, dy)],
        egui::Stroke::new(2.0, color),
    );
}

// ============================================================================
// Papyrus Card Frame
// ============================================================================

/// Draw a papyrus-style card background with content
#[allow(dead_code)]
pub fn papyrus_frame<R>(
    ui: &mut egui::Ui,
    width: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let outer_margin = 4.0;
    let inner_margin = 12.0;

    // Allocate space for the frame
    let response = egui::Frame::new()
        .fill(egui::Color32::TRANSPARENT)
        .outer_margin(outer_margin)
        .show(ui, |ui| {
            ui.set_min_width(width);

            // Get the rect we'll draw in
            let available_rect = ui.available_rect_before_wrap();

            // Draw shadow first
            let shadow_rect = available_rect.translate(egui::vec2(3.0, 3.0));
            ui.painter().rect_filled(
                shadow_rect,
                4.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 40),
            );

            // Draw the papyrus background with gradient effect
            // Main fill
            ui.painter().rect_filled(available_rect, 4.0, PAPYRUS);

            // Darker edges for depth
            let edge_width = 8.0;
            // Top edge
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    available_rect.min,
                    egui::vec2(available_rect.width(), edge_width),
                ),
                egui::CornerRadius {
                    nw: 4,
                    ne: 4,
                    sw: 0,
                    se: 0,
                },
                egui::Color32::from_rgba_unmultiplied(0xD0, 0xC0, 0xA0, 100),
            );
            // Bottom edge
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(available_rect.min.x, available_rect.max.y - edge_width),
                    egui::vec2(available_rect.width(), edge_width),
                ),
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: 4,
                    se: 4,
                },
                egui::Color32::from_rgba_unmultiplied(0xD0, 0xC0, 0xA0, 100),
            );

            // Draw subtle horizontal fiber lines
            let fiber_color = egui::Color32::from_rgba_unmultiplied(0xC0, 0xB0, 0x90, 30);
            for i in 0..5 {
                let y = available_rect.min.y + (available_rect.height() * (i as f32 + 1.0) / 6.0);
                ui.painter().line_segment(
                    [
                        egui::pos2(available_rect.min.x + 4.0, y),
                        egui::pos2(available_rect.max.x - 4.0, y),
                    ],
                    egui::Stroke::new(1.0, fiber_color),
                );
            }

            // Draw border
            ui.painter().rect_stroke(
                available_rect,
                4.0,
                egui::Stroke::new(2.0, GOLD_DARK),
                egui::epaint::StrokeKind::Outside,
            );

            // Now add the actual content with inner margin
            egui::Frame::new()
                .inner_margin(inner_margin)
                .show(ui, add_contents)
                .inner
        });

    response.inner
}

// ============================================================================
// Ornate Panel Frame
// ============================================================================

/// Draw an ornate panel with title bar and decorative corners
#[allow(dead_code)]
pub fn ornate_panel<R>(
    ui: &mut egui::Ui,
    title: &str,
    size: egui::Vec2,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let title_height = 32.0;
    let corner_accent_size = 12.0;
    let border_width = 3.0;

    let (total_rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

    if ui.is_rect_visible(total_rect) {
        let painter = ui.painter();

        // Draw outer shadow
        let shadow_rect = total_rect.translate(egui::vec2(4.0, 4.0));
        painter.rect_filled(
            shadow_rect,
            8.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80),
        );

        // Draw main panel background (dark brown)
        painter.rect_filled(
            total_rect,
            8.0,
            egui::Color32::from_rgb(30, 25, 20),
        );

        // Draw title bar background
        let title_rect = egui::Rect::from_min_size(
            total_rect.min,
            egui::vec2(total_rect.width(), title_height),
        );
        painter.rect_filled(
            title_rect,
            egui::CornerRadius {
                nw: 8,
                ne: 8,
                sw: 0,
                se: 0,
            },
            GOLD_DARK,
        );

        // Draw title text
        painter.text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            title,
            egui::FontId::proportional(20.0),
            PAPYRUS,
        );

        // Draw double border
        painter.rect_stroke(
            total_rect,
            8.0,
            egui::Stroke::new(border_width, GOLD_OUTLINE),
            egui::epaint::StrokeKind::Outside,
        );
        painter.rect_stroke(
            total_rect.shrink(border_width + 1.0),
            6.0,
            egui::Stroke::new(1.0, GOLD_DARK),
            egui::epaint::StrokeKind::Outside,
        );

        // Draw decorative corner diamonds
        let corners = [
            total_rect.left_top(),
            total_rect.right_top(),
            total_rect.left_bottom(),
            total_rect.right_bottom(),
        ];

        for corner in corners {
            draw_corner_diamond(painter, corner, corner_accent_size, GOLD_LIGHT);
        }

        // Draw separator line below title
        painter.line_segment(
            [
                egui::pos2(total_rect.min.x + border_width, total_rect.min.y + title_height),
                egui::pos2(total_rect.max.x - border_width, total_rect.min.y + title_height),
            ],
            egui::Stroke::new(2.0, GOLD_OUTLINE),
        );
    }

    // Create content area below title
    let content_rect = egui::Rect::from_min_max(
        total_rect.min + egui::vec2(border_width + 8.0, title_height + 8.0),
        total_rect.max - egui::vec2(border_width + 8.0, border_width + 8.0),
    );

    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(content_rect)
            .layout(egui::Layout::top_down(egui::Align::LEFT)),
    );

    add_contents(&mut content_ui)
}

/// Draw a decorative diamond at a corner
#[allow(dead_code)]
fn draw_corner_diamond(painter: &egui::Painter, corner: egui::Pos2, size: f32, color: egui::Color32) {
    let half = size / 2.0;
    let points = vec![
        corner + egui::vec2(half, 0.0),
        corner + egui::vec2(size, half),
        corner + egui::vec2(half, size),
        corner + egui::vec2(0.0, half),
    ];

    painter.add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::new(1.0, GOLD_OUTLINE),
    ));
}

// ============================================================================
// Golden Tab Button
// ============================================================================

/// Draw a golden tab button for navigation
/// Returns true if clicked
pub fn gold_tab(ui: &mut egui::Ui, text: &str, selected: bool) -> egui::Response {
    let size = egui::vec2(90.0, 32.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        let (bg_color, text_color, border_color) = if selected {
            (GOLD_LIGHT, GOLD_OUTLINE, GOLD_OUTLINE)
        } else if response.hovered() {
            (STONE_LIGHT, PAPYRUS, GOLD_DARK)
        } else {
            (STONE, PAPYRUS_DARK, STONE_DARK)
        };

        // Draw tab shape (rounded top, flat bottom when selected)
        let rounding = if selected {
            egui::CornerRadius {
                nw: 6,
                ne: 6,
                sw: 0,
                se: 0,
            }
        } else {
            egui::CornerRadius::same(4)
        };

        // Shadow for unselected tabs
        if !selected {
            let shadow_rect = rect.translate(egui::vec2(1.0, 1.0));
            painter.rect_filled(
                shadow_rect,
                rounding,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 40),
            );
        }

        painter.rect_filled(rect, rounding, bg_color);
        painter.rect_stroke(
            rect,
            rounding,
            egui::Stroke::new(1.5, border_color),
            egui::epaint::StrokeKind::Outside,
        );

        // Text
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(13.0),
            text_color,
        );
    }

    response
}

// ============================================================================
// Font Configuration
// ============================================================================

/// Resource to track if fonts have been configured
#[derive(bevy::prelude::Resource, Default)]
pub struct FontsConfigured(pub bool);

/// Configure egui to use the Aleo font as the default proportional font.
/// This system runs every frame but only configures fonts once.
pub fn configure_fonts(mut contexts: EguiContexts, mut configured: bevy::prelude::ResMut<FontsConfigured>) {
    if configured.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut fonts = egui::FontDefinitions::default();

    // Load Aleo variable font
    fonts.font_data.insert(
        "Aleo".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Aleo-Variable.ttf"
        ))),
    );

    // Set Aleo as the primary proportional font
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "Aleo".to_owned());

    ctx.set_fonts(fonts);
    configured.0 = true;
}
