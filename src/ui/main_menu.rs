use crate::game::ai::{AiConfig, AiDifficulty};
use crate::game::state::GameState;
use crate::ui::characters::{draw_avatar, CharacterId};
use crate::ui::hud::UiState;
use crate::ui::player_setup::PlayerSetupConfig;
use crate::ui::rules::{draw_rules_ui, RulesState};
use crate::ui::theme::{
    desert_button, desert_button_enabled, desert_combobox, desert_toggle, DesertButtonStyle,
    PLAYER_COLORS, STONE_DARK,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashSet;

// Colors for the pyramid background
const SKY_BLUE: egui::Color32 = egui::Color32::from_rgb(0x87, 0xCE, 0xEB);
const SAND_COLOR: egui::Color32 = egui::Color32::from_rgb(0xED, 0xC9, 0x9A);
const PYRAMID_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);
const PYRAMID_DARK: egui::Color32 = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);
const PYRAMID_OUTLINE: egui::Color32 = egui::Color32::from_rgb(0x6B, 0x4A, 0x1A);
const SUN_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xD7, 0x00);

// Colors for walking camels - sandy colored with 4-layer style like in-game camels
const CAMEL_MAIN: egui::Color32 = egui::Color32::from_rgb(0xC4, 0xA0, 0x6A); // Sandy tan
const CAMEL_BORDER: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x6A, 0x3A); // Darker border
const CAMEL_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(0xE0, 0xC8, 0x9A); // Lighter highlight

pub fn main_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut config: ResMut<PlayerSetupConfig>,
    mut ai_config: ResMut<AiConfig>,
    ui_state: Res<UiState>,
    time: Res<Time>,
    mut rules_state: ResMut<RulesState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let is_mobile = !ui_state.use_side_panels;
    let elapsed = time.elapsed_secs();

    // Draw rules UI if open (on top of everything)
    draw_rules_ui(ctx, &mut rules_state, is_mobile, time.delta_secs());

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            // Draw pyramid background behind everything
            let rect = ui.available_rect_before_wrap();
            draw_pyramid_background(ui.painter(), rect, elapsed);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(if is_mobile { 20.0 } else { 40.0 });

                    // Responsive title size
                    let title_size = if is_mobile { 32.0 } else { 48.0 };
                    ui.heading(
                        egui::RichText::new("CAMEL UP")
                            .size(title_size)
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("A camel racing board game")
                            .color(egui::Color32::WHITE),
                    );

                    ui.add_space(if is_mobile { 15.0 } else { 30.0 });

                    // Player Setup Section
                    egui::Frame::new().inner_margin(12.0).show(ui, |ui| {
                        // Responsive width
                        if !is_mobile {
                            ui.set_min_width(400.0);
                        }

                        // Player count controls
                        let compact_style = DesertButtonStyle::compact();
                        ui.horizontal(|ui| {
                            ui.add_space(if is_mobile { 8.0 } else { 20.0 });
                            ui.label(
                                egui::RichText::new(format!("{} players", config.players.len()))
                                    .size(16.0)
                                    .color(egui::Color32::WHITE),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_space(if is_mobile { 8.0 } else { 20.0 });
                                    let can_add =
                                        config.players.len() < PlayerSetupConfig::MAX_PLAYERS;
                                    if desert_button_enabled(ui, "+", &compact_style, can_add)
                                        .clicked()
                                        && can_add
                                    {
                                        config.add_player();
                                    }

                                    let can_remove =
                                        config.players.len() > PlayerSetupConfig::MIN_PLAYERS;
                                    if desert_button_enabled(ui, "-", &compact_style, can_remove)
                                        .clicked()
                                        && can_remove
                                    {
                                        config.remove_player();
                                    }
                                },
                            );
                        });

                        ui.add_space(5.0);

                        // Player list
                        egui::ScrollArea::vertical()
                            .max_height(220.0)
                            .show(ui, |ui| {
                                for i in 0..config.players.len() {
                                    let player_color = PLAYER_COLORS[config.players[i].color_index % PLAYER_COLORS.len()];
                                    let character_id = config.players[i].character_id;

                                    // Calculate used characters (for cycling)
                                    let used: HashSet<CharacterId> = config
                                        .players
                                        .iter()
                                        .enumerate()
                                        .filter(|(idx, _)| *idx != i)
                                        .map(|(_, p)| p.character_id)
                                        .collect();

                                    // Use fixed-height row with centered vertical alignment
                                    let row_height = 44.0;
                                    ui.allocate_ui(
                                        egui::vec2(ui.available_width(), row_height),
                                        |ui| {
                                            ui.with_layout(
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    ui.add_space(10.0);

                                                    // Character avatar (tap to cycle)
                                                    let avatar_size = 40.0;
                                                    let (rect, response) = ui.allocate_exact_size(
                                                        egui::vec2(avatar_size, avatar_size),
                                                        egui::Sense::click(),
                                                    );
                                                    draw_avatar(
                                                        ui.painter(),
                                                        rect,
                                                        character_id,
                                                        Some(player_color),
                                                    );

                                                    // Cycle on click
                                                    if response.clicked() {
                                                        let current_idx = character_id as usize;
                                                        for offset in 1..=8 {
                                                            let next = CharacterId::from_index(
                                                                (current_idx + offset) % 8,
                                                            );
                                                            if !used.contains(&next) {
                                                                config.players[i].character_id = next;
                                                                // Update name if AI and not manually edited
                                                                if config.players[i].is_ai && !config.players[i].name_edited {
                                                                    config.players[i].name = next.random_name();
                                                                }
                                                                break;
                                                            }
                                                        }
                                                    }

                                                    ui.add_space(10.0);

                                                    // Calculate remaining width for flexible name input
                                                    // Reserve space for: toggle (100) + spacing (8) + right padding (10)
                                                    let toggle_width = 100.0;
                                                    let reserved_width = toggle_width + 8.0 + 10.0;
                                                    let _available =
                                                        ui.available_width() - reserved_width;
                                                    let name_width = 100.0;

                                                    // Name input - themed style, flexible width
                                                    // Store previous name to detect manual edits
                                                    let prev_name = config.players[i].name.clone();
                                                    let name = &mut config.players[i].name;
                                                    let text_edit =
                                                        egui::TextEdit::singleline(name)
                                                            .desired_width(name_width)
                                                            .font(egui::FontId::proportional(14.0))
                                                            .text_color(egui::Color32::WHITE);
                                                    let response = ui.scope(|ui| {
                                                        ui.visuals_mut().extreme_bg_color =
                                                            STONE_DARK;
                                                        ui.add(text_edit)
                                                    }).inner;
                                                    // If user changed the name, mark it as edited
                                                    if response.changed() && config.players[i].name != prev_name {
                                                        config.players[i].name_edited = true;
                                                    }

                                                    ui.add_space(8.0);

                                                    // Human/AI toggle
                                                    let is_ai = config.players[i].is_ai;
                                                    let (left_clicked, right_clicked) =
                                                        desert_toggle(
                                                            ui,
                                                            ("player_type", i),
                                                            is_ai,
                                                            "Human",
                                                            "AI",
                                                        );
                                                    if left_clicked {
                                                        config.set_player_is_ai(i, false);
                                                    }
                                                    if right_clicked {
                                                        config.set_player_is_ai(i, true);
                                                    }

                                                    ui.add_space(10.0);
                                                },
                                            );
                                        },
                                    );
                                    ui.add_space(2.0);
                                }
                            });

                        ui.add_space(10.0);
                        // Custom themed separator
                        let separator_rect = ui.available_rect_before_wrap();
                        let separator_width = separator_rect.width();
                        let (sep_rect, _) = ui.allocate_exact_size(
                            egui::vec2(separator_width, 2.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().line_segment(
                            [sep_rect.left_center(), sep_rect.right_center()],
                            egui::Stroke::new(
                                1.5,
                                egui::Color32::from_rgba_unmultiplied(0x8A, 0x7B, 0x6A, 150),
                            ), // STONE with transparency
                        );
                        ui.add_space(5.0);

                        // AI Difficulty selector (only show if there are AI players)
                        let has_ai_players = config.players.iter().any(|p| p.is_ai);
                        if has_ai_players {
                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                ui.label(
                                    egui::RichText::new("AI Difficulty:")
                                        .color(egui::Color32::WHITE),
                                );
                                ui.add_space(8.0);

                                let options = [
                                    AiDifficulty::Random,
                                    AiDifficulty::Basic,
                                    AiDifficulty::Smart,
                                ];
                                let labels = ["Random", "Basic", "Smart"];
                                desert_combobox(
                                    ui,
                                    "ai_difficulty",
                                    &mut ai_config.difficulty,
                                    &options,
                                    &labels,
                                );
                            });
                            ui.add_space(5.0);
                        }

                        // Randomize start order toggle
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.checkbox(
                                &mut config.randomize_start_order,
                                egui::RichText::new("Randomize player order")
                                    .color(egui::Color32::WHITE),
                            );
                        });

                        ui.add_space(10.0);
                    });

                    ui.add_space(if is_mobile { 15.0 } else { 30.0 });

                    // Start Game button - larger on mobile for touch
                    let start_style = if is_mobile {
                        DesertButtonStyle {
                            min_size: egui::vec2(280.0, 60.0),
                            corner_radius: 10.0,
                            font_size: 22.0,
                        }
                    } else {
                        DesertButtonStyle::large()
                    };

                    if desert_button(ui, "Start Game", &start_style).clicked() {
                        next_state.set(GameState::Playing);
                    }

                    ui.add_space(if is_mobile { 10.0 } else { 15.0 });

                    // How to Play button
                    let medium_style = DesertButtonStyle::medium();
                    if desert_button(ui, "How to Play", &medium_style).clicked() {
                        rules_state.is_open = true;
                    }

                    ui.add_space(if is_mobile { 10.0 } else { 15.0 });

                    // Quit button (hide on mobile/web - users close the browser tab)
                    #[cfg(not(target_arch = "wasm32"))]
                    if desert_button(ui, "Quit", &DesertButtonStyle::small()).clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });
}

/// Draw a walking camel silhouette
/// x, y: position of camel center
/// scale: size multiplier (1.0 = normal size)
/// time: elapsed time for leg animation
/// phase_offset: offset for walking cycle (different camels walk at different phases)
fn draw_walking_camel(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    scale: f32,
    time: f32,
    phase_offset: f32,
    alpha: u8,
) {
    // Camel dimensions (scaled)
    let body_w = 32.0 * scale;
    let body_h = 14.0 * scale;
    let hump_w = 12.0 * scale;
    let hump_h = 10.0 * scale;
    let neck_w = 6.0 * scale;
    let neck_h = 14.0 * scale;
    let head_w = 10.0 * scale;
    let head_h = 7.0 * scale;
    let leg_w = 4.0 * scale;
    let leg_h = 12.0 * scale;

    // Walking animation - legs swing with sine wave
    let walk_speed = 3.0;
    let leg_amplitude = 4.0 * scale;

    // Four legs with different phase offsets for walking gait
    let leg_phases = [
        0.0,
        std::f32::consts::PI,
        std::f32::consts::PI * 0.5,
        std::f32::consts::PI * 1.5,
    ];

    // Colors with alpha for distance fading
    let shadow_color =
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, (40.0 * alpha as f32 / 255.0) as u8);
    let border_color = egui::Color32::from_rgba_unmultiplied(
        CAMEL_BORDER.r(),
        CAMEL_BORDER.g(),
        CAMEL_BORDER.b(),
        alpha,
    );
    let main_color = egui::Color32::from_rgba_unmultiplied(
        CAMEL_MAIN.r(),
        CAMEL_MAIN.g(),
        CAMEL_MAIN.b(),
        alpha,
    );
    let highlight_color = egui::Color32::from_rgba_unmultiplied(
        CAMEL_HIGHLIGHT.r(),
        CAMEL_HIGHLIGHT.g(),
        CAMEL_HIGHLIGHT.b(),
        alpha,
    );

    let shadow_offset = 2.0 * scale;
    let border_expand = 1.5 * scale;

    // Helper to draw a 4-layer rect: shadow, border, main, optional highlight
    let draw_layered_rect = |painter: &egui::Painter,
                             cx: f32,
                             cy: f32,
                             w: f32,
                             h: f32,
                             rounding: f32,
                             with_highlight: bool| {
        // Shadow layer
        painter.rect_filled(
            egui::Rect::from_center_size(
                egui::pos2(cx + shadow_offset, cy + shadow_offset),
                egui::vec2(w, h),
            ),
            rounding,
            shadow_color,
        );
        // Border layer (slightly larger)
        painter.rect_filled(
            egui::Rect::from_center_size(
                egui::pos2(cx, cy),
                egui::vec2(w + border_expand, h + border_expand),
            ),
            rounding,
            border_color,
        );
        // Main color layer
        painter.rect_filled(
            egui::Rect::from_center_size(egui::pos2(cx, cy), egui::vec2(w, h)),
            rounding,
            main_color,
        );
        // Highlight layer (smaller, offset up-left)
        if with_highlight {
            let highlight_w = w * 0.5;
            let highlight_h = h * 0.4;
            painter.rect_filled(
                egui::Rect::from_center_size(
                    egui::pos2(cx - w * 0.15, cy - h * 0.2),
                    egui::vec2(highlight_w, highlight_h),
                ),
                rounding * 0.5,
                highlight_color,
            );
        }
    };

    // Draw legs first (behind body) with walking animation
    let leg_base_y = y + body_h / 2.0 + leg_h / 2.0;
    let leg_x_positions = [
        x - body_w / 2.0 + 5.0 * scale,  // Back left
        x - body_w / 2.0 + 10.0 * scale, // Back right
        x + body_w / 2.0 - 10.0 * scale, // Front left
        x + body_w / 2.0 - 5.0 * scale,  // Front right
    ];

    for (i, &leg_x) in leg_x_positions.iter().enumerate() {
        let leg_offset =
            ((time * walk_speed + phase_offset + leg_phases[i]).sin() * leg_amplitude).abs();
        let current_leg_h = leg_h - leg_offset;
        let leg_cy = leg_base_y - leg_offset / 2.0;
        draw_layered_rect(
            painter,
            leg_x,
            leg_cy,
            leg_w,
            current_leg_h,
            1.0 * scale,
            false,
        );
    }

    // Draw body
    draw_layered_rect(painter, x, y, body_w, body_h, 2.0 * scale, true);

    // Draw hump (with highlight)
    let hump_cx = x - 2.0 * scale;
    let hump_cy = y - body_h / 2.0 - hump_h / 2.0 + 2.0 * scale;
    draw_layered_rect(painter, hump_cx, hump_cy, hump_w, hump_h, 4.0 * scale, true);

    // Draw neck
    let neck_cx = x + body_w / 2.0 - 2.0 * scale;
    let neck_cy = y - body_h / 2.0 - neck_h / 2.0 + 3.0 * scale;
    draw_layered_rect(
        painter,
        neck_cx,
        neck_cy,
        neck_w,
        neck_h,
        1.0 * scale,
        false,
    );

    // Draw head (with highlight)
    let head_cx = x + body_w / 2.0 + 2.0 * scale;
    let head_cy = y - body_h / 2.0 - neck_h + 2.0 * scale;
    draw_layered_rect(painter, head_cx, head_cy, head_w, head_h, 2.0 * scale, true);
}

/// Draw the pyramid scene as a background decoration
fn draw_pyramid_background(painter: &egui::Painter, rect: egui::Rect, time: f32) {
    // Draw sky background
    painter.rect_filled(rect, 0.0, SKY_BLUE);

    // Calculate horizon line (roughly 80% down)
    let horizon_y = rect.top() + rect.height() * 0.80;

    // Draw sand (desert floor)
    let sand_rect = egui::Rect::from_min_max(egui::pos2(rect.left(), horizon_y), rect.max);
    painter.rect_filled(sand_rect, 0.0, SAND_COLOR);

    // Draw sun in upper right
    let sun_radius = rect.width() * 0.06;
    let sun_center = egui::pos2(
        rect.right() - rect.width() * 0.15,
        rect.top() + rect.height() * 0.15,
    );
    painter.circle_filled(sun_center, sun_radius, SUN_COLOR);

    // Draw pyramid (large, centered)
    let pyramid_width = rect.width() * 0.5;
    // Use width-based height to maintain aspect ratio on tall screens
    let pyramid_height = pyramid_width * 0.7;
    let pyramid_center_x = rect.center().x;
    let pyramid_base_y = horizon_y;
    let pyramid_apex_y = pyramid_base_y - pyramid_height;

    let apex = egui::pos2(pyramid_center_x, pyramid_apex_y);
    let base_left = egui::pos2(pyramid_center_x - pyramid_width / 2.0, pyramid_base_y);
    let base_right = egui::pos2(pyramid_center_x + pyramid_width / 2.0, pyramid_base_y);

    // Draw pyramid shadow side (left)
    let left_triangle = vec![
        apex,
        base_left,
        egui::pos2(pyramid_center_x, pyramid_base_y),
    ];
    painter.add(egui::Shape::convex_polygon(
        left_triangle,
        PYRAMID_DARK,
        egui::Stroke::NONE,
    ));

    // Draw pyramid lit side (right)
    let right_triangle = vec![
        apex,
        egui::pos2(pyramid_center_x, pyramid_base_y),
        base_right,
    ];
    painter.add(egui::Shape::convex_polygon(
        right_triangle,
        PYRAMID_LIGHT,
        egui::Stroke::NONE,
    ));

    // Draw pyramid outline
    let outline_stroke = egui::Stroke::new(2.0, PYRAMID_OUTLINE);
    painter.line_segment([apex, base_left], outline_stroke);
    painter.line_segment([apex, base_right], outline_stroke);
    painter.line_segment([base_left, base_right], outline_stroke);

    // Draw a second smaller pyramid in the background (left)
    let small_pyramid_width = pyramid_width * 0.35;
    // Use same aspect ratio as main pyramid
    let small_pyramid_height = small_pyramid_width * 0.7;
    let small_center_x = rect.left() + rect.width() * 0.2;
    let small_base_y = horizon_y;
    let small_apex_y = small_base_y - small_pyramid_height;

    let small_apex = egui::pos2(small_center_x, small_apex_y);
    let small_base_left = egui::pos2(small_center_x - small_pyramid_width / 2.0, small_base_y);
    let small_base_right = egui::pos2(small_center_x + small_pyramid_width / 2.0, small_base_y);

    // Faded colors for distant pyramid
    let distant_dark = egui::Color32::from_rgba_unmultiplied(0xA0, 0x7A, 0x30, 180);
    let distant_light = egui::Color32::from_rgba_unmultiplied(0xD4, 0xA8, 0x4B, 180);

    let small_left = vec![
        small_apex,
        small_base_left,
        egui::pos2(small_center_x, small_base_y),
    ];
    painter.add(egui::Shape::convex_polygon(
        small_left,
        distant_dark,
        egui::Stroke::NONE,
    ));

    let small_right = vec![
        small_apex,
        egui::pos2(small_center_x, small_base_y),
        small_base_right,
    ];
    painter.add(egui::Shape::convex_polygon(
        small_right,
        distant_light,
        egui::Stroke::NONE,
    ));

    // Draw walking camels on the sand
    let screen_width = rect.width();
    let camel_speed = 25.0; // pixels per second

    // Camel caravan - 3 camels at different positions and sizes
    let camels = [
        // (phase_offset, scale, y_offset from horizon, speed_mult)
        (0.0, 0.8, 20.0, 1.0),  // Closer, larger
        (2.0, 0.5, 8.0, 0.8),   // Farther, smaller (near horizon)
        (4.5, 0.65, 14.0, 0.9), // Middle distance
    ];

    for (phase, scale, y_off, speed_mult) in camels {
        // Calculate x position - wraps around screen
        let travel_distance = screen_width + 100.0 * scale;
        let x_offset = (time * camel_speed * speed_mult + phase * 100.0) % travel_distance;
        let camel_x = rect.left() - 50.0 * scale + x_offset;
        let camel_y = horizon_y + y_off;

        // Use alpha for distance fading (smaller scale = farther away = more faded)
        let alpha = ((200.0 * scale) as u8).max(100);

        draw_walking_camel(painter, camel_x, camel_y, scale, time, phase, alpha);
    }
}
