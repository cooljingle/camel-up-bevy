use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::state::GameState;
use crate::game::ai::{AiConfig, AiDifficulty};
use crate::ui::player_setup::PlayerSetupConfig;
use crate::ui::characters::{CharacterId, draw_avatar};

// Colors for the pyramid background
const SKY_BLUE: egui::Color32 = egui::Color32::from_rgb(0x87, 0xCE, 0xEB);
const SAND_COLOR: egui::Color32 = egui::Color32::from_rgb(0xED, 0xC9, 0x9A);
const PYRAMID_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xD4, 0xA8, 0x4B);
const PYRAMID_DARK: egui::Color32 = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);
const PYRAMID_OUTLINE: egui::Color32 = egui::Color32::from_rgb(0x6B, 0x4A, 0x1A);
const SUN_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xD7, 0x00);

// Player colors (same as hud.rs)
const PLAYER_COLORS: [egui::Color32; 8] = [
    egui::Color32::from_rgb(66, 133, 244),   // Blue
    egui::Color32::from_rgb(219, 68, 55),    // Red
    egui::Color32::from_rgb(244, 180, 0),    // Yellow
    egui::Color32::from_rgb(15, 157, 88),    // Green
    egui::Color32::from_rgb(171, 71, 188),   // Purple
    egui::Color32::from_rgb(255, 112, 67),   // Orange
    egui::Color32::from_rgb(0, 172, 193),    // Cyan
    egui::Color32::from_rgb(124, 77, 255),   // Indigo
];

pub fn main_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut config: ResMut<PlayerSetupConfig>,
    mut ai_config: ResMut<AiConfig>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default().show(ctx, |ui| {
        // Draw pyramid background behind everything
        let rect = ui.available_rect_before_wrap();
        draw_pyramid_background(ui.painter(), rect);

        ui.vertical_centered(|ui| {
            ui.add_space(40.0);

            ui.heading(egui::RichText::new("CAMEL UP").size(48.0));
            ui.add_space(10.0);
            ui.label("A game of racing camels and risky bets!");

            ui.add_space(30.0);

            // Player Setup Section
            ui.group(|ui| {
                ui.set_min_width(400.0);

                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new("Players").size(24.0));
                });

                ui.add_space(10.0);

                // Player count controls
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new(format!("{} players", config.players.len())).size(16.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(20.0);
                        let can_add = config.players.len() < PlayerSetupConfig::MAX_PLAYERS;
                        if ui.add_enabled(can_add, egui::Button::new("+").min_size(egui::vec2(28.0, 28.0))).clicked() {
                            config.add_player();
                        }

                        let can_remove = config.players.len() > PlayerSetupConfig::MIN_PLAYERS;
                        if ui.add_enabled(can_remove, egui::Button::new("-").min_size(egui::vec2(28.0, 28.0))).clicked() {
                            config.remove_player();
                        }
                    });
                });

                ui.add_space(10.0);

                // Player list
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for i in 0..config.players.len() {
                            let player_color = PLAYER_COLORS[i % PLAYER_COLORS.len()];
                            let character_id = CharacterId::from_index(i);

                            ui.horizontal(|ui| {
                                ui.add_space(10.0);

                                // Character avatar preview
                                let avatar_size = 36.0;
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(avatar_size, avatar_size), egui::Sense::hover());
                                draw_avatar(ui.painter(), rect, character_id, Some(player_color));

                                ui.add_space(8.0);

                                // Name input
                                let name = &mut config.players[i].name;
                                ui.add(egui::TextEdit::singleline(name).desired_width(120.0));

                                ui.add_space(10.0);

                                // Human/AI toggle as compact buttons
                                let is_ai = config.players[i].is_ai;
                                if ui.selectable_label(!is_ai, "Human").clicked() {
                                    config.players[i].is_ai = false;
                                }
                                if ui.selectable_label(is_ai, "AI").clicked() {
                                    config.players[i].is_ai = true;
                                }
                            });
                            ui.add_space(6.0);
                        }
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);

                // AI Difficulty selector (only show if there are AI players)
                let has_ai_players = config.players.iter().any(|p| p.is_ai);
                if has_ai_players {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.label("AI Difficulty:");

                        egui::ComboBox::from_id_salt("ai_difficulty")
                            .selected_text(match ai_config.difficulty {
                                AiDifficulty::Random => "Random",
                                AiDifficulty::Basic => "Basic",
                                AiDifficulty::Smart => "Smart",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut ai_config.difficulty, AiDifficulty::Random, "Random");
                                ui.selectable_value(&mut ai_config.difficulty, AiDifficulty::Basic, "Basic");
                                ui.selectable_value(&mut ai_config.difficulty, AiDifficulty::Smart, "Smart");
                            });
                    });
                    ui.add_space(5.0);
                }

                // Randomize start order toggle
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.checkbox(&mut config.randomize_start_order, "Randomize player order");
                });

                ui.add_space(10.0);
            });

            ui.add_space(30.0);

            // Start Game button
            if ui.add(egui::Button::new(egui::RichText::new("Start Game").size(24.0))
                .min_size(egui::vec2(200.0, 50.0))).clicked() {
                next_state.set(GameState::Playing);
            }

            ui.add_space(20.0);

            if ui.button(egui::RichText::new("Quit").size(16.0)).clicked() {
                std::process::exit(0);
            }
        });
    });
}

/// Draw the pyramid scene as a background decoration
fn draw_pyramid_background(painter: &egui::Painter, rect: egui::Rect) {
    // Draw sky background
    painter.rect_filled(rect, 0.0, SKY_BLUE);

    // Calculate horizon line (roughly 80% down)
    let horizon_y = rect.top() + rect.height() * 0.80;

    // Draw sand (desert floor)
    let sand_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left(), horizon_y),
        rect.max,
    );
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
    let pyramid_height = rect.height() * 0.55;
    let pyramid_center_x = rect.center().x;
    let pyramid_base_y = horizon_y;
    let pyramid_apex_y = pyramid_base_y - pyramid_height;

    let apex = egui::pos2(pyramid_center_x, pyramid_apex_y);
    let base_left = egui::pos2(pyramid_center_x - pyramid_width / 2.0, pyramid_base_y);
    let base_right = egui::pos2(pyramid_center_x + pyramid_width / 2.0, pyramid_base_y);

    // Draw pyramid shadow side (left)
    let left_triangle = vec![apex, base_left, egui::pos2(pyramid_center_x, pyramid_base_y)];
    painter.add(egui::Shape::convex_polygon(
        left_triangle,
        PYRAMID_DARK,
        egui::Stroke::NONE,
    ));

    // Draw pyramid lit side (right)
    let right_triangle = vec![apex, egui::pos2(pyramid_center_x, pyramid_base_y), base_right];
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
    let small_pyramid_height = pyramid_height * 0.4;
    let small_center_x = rect.left() + rect.width() * 0.2;
    let small_base_y = horizon_y;
    let small_apex_y = small_base_y - small_pyramid_height;

    let small_apex = egui::pos2(small_center_x, small_apex_y);
    let small_base_left = egui::pos2(small_center_x - small_pyramid_width / 2.0, small_base_y);
    let small_base_right = egui::pos2(small_center_x + small_pyramid_width / 2.0, small_base_y);

    // Faded colors for distant pyramid
    let distant_dark = egui::Color32::from_rgba_unmultiplied(0xA0, 0x7A, 0x30, 180);
    let distant_light = egui::Color32::from_rgba_unmultiplied(0xD4, 0xA8, 0x4B, 180);

    let small_left = vec![small_apex, small_base_left, egui::pos2(small_center_x, small_base_y)];
    painter.add(egui::Shape::convex_polygon(small_left, distant_dark, egui::Stroke::NONE));

    let small_right = vec![small_apex, egui::pos2(small_center_x, small_base_y), small_base_right];
    painter.add(egui::Shape::convex_polygon(small_right, distant_light, egui::Stroke::NONE));
}
