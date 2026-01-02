use bevy::prelude::*;
use bevy_egui::egui;
use crate::components::CamelColor;
use crate::ui::hud::{draw_camel_silhouette, draw_mini_leg_bet_card, draw_pyramid_token_icon};
use crate::ui::theme::{desert_button, gold_tab, DesertButtonStyle, camel_color_to_egui};

// Desert theme colors
const SAND_COLOR: egui::Color32 = egui::Color32::from_rgb(0xED, 0xC9, 0x9A);
const PYRAMID_DARK: egui::Color32 = egui::Color32::from_rgb(0xA0, 0x7A, 0x30);
const PYRAMID_OUTLINE: egui::Color32 = egui::Color32::from_rgb(0x6B, 0x4A, 0x1A);
const MODAL_BG: egui::Color32 = egui::Color32::from_rgb(30, 25, 20);
const OASIS_GREEN: egui::Color32 = egui::Color32::from_rgb(80, 160, 80);
const MIRAGE_ORANGE: egui::Color32 = egui::Color32::from_rgb(200, 150, 80);

/// Current section/tab in the rules viewer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RulesSection {
    #[default]
    Overview,
    CamelMovement,
    Betting,
    DesertTiles,
    Scoring,
}

impl RulesSection {
    pub fn all() -> [RulesSection; 5] {
        [
            RulesSection::Overview,
            RulesSection::CamelMovement,
            RulesSection::Betting,
            RulesSection::DesertTiles,
            RulesSection::Scoring,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            RulesSection::Overview => "Overview",
            RulesSection::CamelMovement => "Movement",
            RulesSection::Betting => "Betting",
            RulesSection::DesertTiles => "Desert Tiles",
            RulesSection::Scoring => "Scoring",
        }
    }
}

/// Main resource for rules UI state
#[derive(Resource, Default)]
pub struct RulesState {
    pub is_open: bool,
    pub current_section: RulesSection,
    pub demo_elapsed: f32,
    #[allow(dead_code)]
    pub expanded_subsections: [bool; 8],
}

/// Draw the rules modal UI
pub fn draw_rules_ui(
    ctx: &egui::Context,
    rules_state: &mut RulesState,
    is_mobile: bool,
    time_delta: f32,
) {
    if !rules_state.is_open {
        return;
    }

    // Update demo animation timer
    rules_state.demo_elapsed += time_delta;
    if rules_state.demo_elapsed > 8.0 {
        rules_state.demo_elapsed = 0.0;
    }

    // Dark overlay behind modal
    egui::Area::new(egui::Id::new("rules_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let screen_rect = ctx.input(|i| i.viewport_rect());
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
        });

    // Main rules panel
    let screen_rect = ctx.input(|i| i.viewport_rect());
    let panel_size = egui::vec2(screen_rect.width() * 0.95, screen_rect.height() * 0.90);

    egui::Area::new(egui::Id::new("rules_panel"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(MODAL_BG)
                .corner_radius(egui::CornerRadius::same(16))
                .inner_margin(egui::Margin::same(if is_mobile { 16 } else { 24 }))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 4,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
                })
                .show(ui, |ui| {
                    ui.set_min_size(panel_size);
                    ui.set_max_size(panel_size);

                    // Title
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("How to Play").size(28.0).color(egui::Color32::WHITE));
                    });
                    ui.add_space(12.0);

                    draw_mobile_layout(ui, rules_state);

                    ui.add_space(12.0);

                    // Close button
                    ui.vertical_centered(|ui| {
                        if desert_button(ui, "Close", &DesertButtonStyle::medium()).clicked() {
                            rules_state.is_open = false;
                        }
                    });
                });
        });
}

fn draw_mobile_layout(ui: &mut egui::Ui, rules_state: &mut RulesState) {
    // Horizontal tab bar at top using gold_tab theme
    ui.horizontal_wrapped(|ui| {
        for section in RulesSection::all() {
            let selected = rules_state.current_section == section;
            if gold_tab(ui, section.name(), selected).clicked() {
                rules_state.current_section = section;
                rules_state.demo_elapsed = 0.0;
            }
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Full-width content
    egui::ScrollArea::vertical()
        .max_height(350.0)
        .show(ui, |ui| {
            draw_section_content(ui, rules_state, true);
        });
}

fn draw_section_content(ui: &mut egui::Ui, rules_state: &mut RulesState, is_mobile: bool) {
    match rules_state.current_section {
        RulesSection::Overview => draw_overview_section(ui, is_mobile),
        RulesSection::CamelMovement => draw_movement_section(ui, rules_state, is_mobile),
        RulesSection::Betting => draw_betting_section(ui),
        RulesSection::DesertTiles => draw_desert_tiles_section(ui),
        RulesSection::Scoring => draw_scoring_section(ui),
    }
}

// ============================================================================
// Overview Section
// ============================================================================

fn draw_overview_section(ui: &mut egui::Ui, is_mobile: bool) {
    ui.heading(egui::RichText::new("Welcome to Camel Up!").size(20.0).color(egui::Color32::WHITE));
    ui.add_space(12.0);

    ui.label(egui::RichText::new("Be the richest player when a camel crosses the finish line!")
        .size(16.0).color(egui::Color32::from_rgb(255, 215, 0)));
    ui.add_space(16.0);

    ui.label(egui::RichText::new(
        "In Camel Up, you and your fellow players bet on a crazy camel race around \
        a desert pyramid. The camels stack on top of each other, creating \
        unpredictable outcomes. Will your favorite camel come out on top?"
    ).size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.add_space(20.0);

    // Action icons
    ui.heading(egui::RichText::new("On Your Turn").size(18.0).color(egui::Color32::WHITE));
    ui.add_space(12.0);

    let icon_size = if is_mobile { 50.0 } else { 60.0 };
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = if is_mobile { 10.0 } else { 16.0 };

        // Roll Dice action
        draw_action_card(ui, icon_size, "Roll Dice", egui::Color32::from_rgb(100, 150, 200), |painter, rect| {
            // Draw dice
            let dice_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(30.0, 30.0));
            painter.rect_filled(dice_rect, 4.0, egui::Color32::WHITE);
            painter.rect_stroke(dice_rect, 4.0, egui::Stroke::new(2.0, egui::Color32::DARK_GRAY), egui::epaint::StrokeKind::Outside);
            // Pips
            painter.circle_filled(dice_rect.center(), 4.0, egui::Color32::BLACK);
            painter.circle_filled(dice_rect.center() + egui::vec2(-8.0, -8.0), 3.0, egui::Color32::BLACK);
            painter.circle_filled(dice_rect.center() + egui::vec2(8.0, 8.0), 3.0, egui::Color32::BLACK);
        });

        // Leg Bet action
        draw_action_card(ui, icon_size, "Leg Bet", camel_color_to_egui(CamelColor::Blue), |painter, rect| {
            let card_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(28.0, 36.0));
            painter.rect_filled(card_rect, 3.0, egui::Color32::from_rgb(245, 235, 215));
            draw_camel_silhouette(painter, card_rect.shrink(4.0), camel_color_to_egui(CamelColor::Blue), egui::Color32::DARK_GRAY);
        });

        // Desert Tile action
        draw_action_card(ui, icon_size, "Place Tile", OASIS_GREEN, |painter, rect| {
            let tile_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(32.0, 32.0));
            painter.rect_filled(tile_rect, 4.0, OASIS_GREEN);
            painter.rect_stroke(tile_rect, 4.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(40, 100, 40)), egui::epaint::StrokeKind::Outside);
            painter.text(tile_rect.center(), egui::Align2::CENTER_CENTER, "+1",
                egui::FontId::proportional(14.0), egui::Color32::WHITE);
        });

        // Race Bet action
        draw_action_card(ui, icon_size, "Race Bet", egui::Color32::from_rgb(200, 150, 50), |painter, rect| {
            // Trophy shape
            let center = rect.center();
            painter.rect_filled(
                egui::Rect::from_center_size(center + egui::vec2(0.0, 4.0), egui::vec2(20.0, 16.0)),
                4.0, egui::Color32::GOLD
            );
            painter.rect_filled(
                egui::Rect::from_center_size(center + egui::vec2(0.0, 14.0), egui::vec2(12.0, 6.0)),
                2.0, egui::Color32::GOLD
            );
            // Handles
            painter.circle_stroke(center + egui::vec2(-12.0, 0.0), 6.0, egui::Stroke::new(2.0, egui::Color32::GOLD));
            painter.circle_stroke(center + egui::vec2(12.0, 0.0), 6.0, egui::Stroke::new(2.0, egui::Color32::GOLD));
        });
    });

    ui.add_space(20.0);

    // Key concepts
    ui.heading(egui::RichText::new("Key Concepts").size(18.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    let bullet_color = egui::Color32::from_rgb(255, 215, 0);
    let highlight_blue = egui::Color32::from_rgb(100, 200, 255);
    let highlight_green = egui::Color32::from_rgb(100, 255, 100);
    let highlight_orange = egui::Color32::from_rgb(255, 180, 100);
    let highlight_red = egui::Color32::from_rgb(255, 100, 100);

    // Use LayoutJob for mixed formatting in a single label
    let mut job = egui::text::LayoutJob::default();
    job.wrap = egui::text::TextWrapping {
        max_width: ui.available_width(),
        ..Default::default()
    };

    // Bullet 1: Camels stack
    job.append("• ", 0.0, egui::TextFormat { color: bullet_color, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("Camels ", 0.0, egui::TextFormat { color: egui::Color32::WHITE, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("stack", 0.0, egui::TextFormat { color: highlight_blue, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append(" when they land on the same space\n", 0.0, egui::TextFormat { color: egui::Color32::LIGHT_GRAY, font_id: egui::FontId::proportional(14.0), ..Default::default() });

    // Bullet 2: On top is ahead
    job.append("• ", 0.0, egui::TextFormat { color: bullet_color, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("The camel ", 0.0, egui::TextFormat { color: egui::Color32::LIGHT_GRAY, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("on top", 0.0, egui::TextFormat { color: highlight_green, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append(" is considered ", 0.0, egui::TextFormat { color: egui::Color32::LIGHT_GRAY, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("ahead", 0.0, egui::TextFormat { color: highlight_green, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("\n", 0.0, egui::TextFormat::default());

    // Bullet 3: Leg ends
    job.append("• ", 0.0, egui::TextFormat { color: bullet_color, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("A ", 0.0, egui::TextFormat { color: egui::Color32::LIGHT_GRAY, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("leg", 0.0, egui::TextFormat { color: highlight_orange, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append(" ends when 5 dice have been rolled (including crazy camels)\n", 0.0, egui::TextFormat { color: egui::Color32::LIGHT_GRAY, font_id: egui::FontId::proportional(14.0), ..Default::default() });

    // Bullet 4: Game ends
    job.append("• ", 0.0, egui::TextFormat { color: bullet_color, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("The ", 0.0, egui::TextFormat { color: egui::Color32::LIGHT_GRAY, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append("game", 0.0, egui::TextFormat { color: highlight_red, font_id: egui::FontId::proportional(14.0), ..Default::default() });
    job.append(" ends when a camel crosses space 16", 0.0, egui::TextFormat { color: egui::Color32::LIGHT_GRAY, font_id: egui::FontId::proportional(14.0), ..Default::default() });

    ui.label(job);
}

fn draw_action_card(
    ui: &mut egui::Ui,
    size: f32,
    label: &str,
    accent_color: egui::Color32,
    draw_icon: impl FnOnce(&egui::Painter, egui::Rect),
) {
    ui.vertical(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());

        // Card background
        ui.painter().rect_filled(rect, 8.0, egui::Color32::from_rgb(50, 45, 40));
        ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(2.0, accent_color), egui::epaint::StrokeKind::Outside);

        // Draw custom icon
        draw_icon(ui.painter(), rect);

        // Label below
        ui.add_space(4.0);
        ui.label(egui::RichText::new(label).size(11.0).color(egui::Color32::LIGHT_GRAY));
    });
}

// ============================================================================
// Camel Movement Section
// ============================================================================

fn draw_movement_section(ui: &mut egui::Ui, rules_state: &mut RulesState, is_mobile: bool) {
    ui.heading(egui::RichText::new("Camel Movement").size(20.0).color(egui::Color32::WHITE));
    ui.add_space(12.0);

    // Stacking Demo
    ui.group(|ui| {
        // Don't set min width on mobile - let it be responsive
        if !is_mobile {
            ui.set_min_width(400.0);
        }
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("How Stacking Works").size(16.0).color(egui::Color32::WHITE).strong());
        });
        ui.add_space(8.0);

        draw_stacking_demo(ui, rules_state.demo_elapsed, is_mobile);

        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            if ui.small_button("Reset Demo").clicked() {
                rules_state.demo_elapsed = 0.0;
            }
        });
    });

    ui.add_space(16.0);

    // Rolling the Pyramid Die
    ui.heading(egui::RichText::new("Rolling the Pyramid Die").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.label(egui::RichText::new(
        "When you roll the pyramid, one of the five colored dice is randomly selected \
        and rolled. The matching camel moves that many spaces (1-3) forward."
    ).size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.add_space(8.0);

    // Draw dice
    ui.horizontal(|ui| {
        for color in CamelColor::all() {
            let egui_color = camel_color_to_egui(color);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 4.0, egui_color);
            ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::WHITE), egui::epaint::StrokeKind::Outside);

            let pip_color = if color == CamelColor::Yellow { egui::Color32::BLACK } else { egui::Color32::WHITE };
            ui.painter().circle_filled(rect.center(), 3.0, pip_color);
            ui.painter().circle_filled(rect.center() + egui::vec2(-6.0, -6.0), 2.0, pip_color);
            ui.painter().circle_filled(rect.center() + egui::vec2(6.0, 6.0), 2.0, pip_color);
        }
    });

    ui.add_space(16.0);

    // Stacking rules
    ui.heading(egui::RichText::new("Stacking Rules").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.label(egui::RichText::new("• When a camel lands on an occupied space, it lands ON TOP")
        .size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.label(egui::RichText::new("• When a camel moves, it CARRIES all camels above it")
        .size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.label(egui::RichText::new("• The camel on top of a stack is AHEAD of camels below")
        .size(14.0).color(egui::Color32::from_rgb(100, 255, 100)));

    ui.add_space(16.0);

    // Crazy Camels
    ui.heading(egui::RichText::new("Crazy Camels (Black & White)").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        // Black camel icon
        let (rect, _) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(40, 40, 40));

        // White camel icon
        let (rect2, _) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
        ui.painter().rect_filled(rect2, 4.0, egui::Color32::from_rgb(240, 240, 240));
        ui.painter().rect_stroke(rect2, 4.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::epaint::StrokeKind::Outside);
    });
    ui.add_space(4.0);

    ui.label(egui::RichText::new("• They move BACKWARDS around the track")
        .size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.label(egui::RichText::new("• They land UNDERNEATH other camels (not on top)")
        .size(14.0).color(egui::Color32::from_rgb(255, 150, 150)));
    ui.label(egui::RichText::new("• They don't count for leg bets or race bets")
        .size(14.0).color(egui::Color32::LIGHT_GRAY));
}

fn draw_stacking_demo(ui: &mut egui::Ui, elapsed: f32, is_mobile: bool) {
    // Make width responsive - use available width clamped to reasonable bounds
    let demo_width = if is_mobile {
        ui.available_width().min(400.0).max(260.0)
    } else {
        400.0
    };
    let demo_height = if is_mobile { 140.0 } else { 160.0 };

    let (rect, _) = ui.allocate_exact_size(egui::vec2(demo_width, demo_height), egui::Sense::hover());
    let painter = ui.painter();

    // Background
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(40, 35, 30));

    // Scale factor for responsive sizing (based on 400px reference)
    let scale = demo_width / 400.0;

    // Draw 3 track spaces - scaled
    let space_width = 80.0 * scale;
    let space_height = 40.0 * scale.max(0.8); // Don't shrink height as much
    let track_y = rect.max.y - (50.0 * scale.max(0.8));
    let start_x = rect.min.x + (60.0 * scale);

    let space_gap = 20.0 * scale;
    for i in 0..3 {
        let space_rect = egui::Rect::from_min_size(
            egui::pos2(start_x + (i as f32 * (space_width + space_gap)), track_y),
            egui::vec2(space_width, space_height),
        );
        painter.rect_filled(space_rect, 4.0, SAND_COLOR);
        painter.rect_stroke(space_rect, 4.0, egui::Stroke::new(2.0, PYRAMID_OUTLINE), egui::epaint::StrokeKind::Outside);

        painter.text(
            space_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{}", i + 1),
            egui::FontId::proportional(16.0),
            PYRAMID_DARK,
        );
    }

    // Animation phases (8 seconds total, looping)
    let phase = (elapsed / 2.0) as usize % 4;
    let phase_progress = (elapsed % 2.0) / 2.0;

    let camel_size = egui::vec2(36.0 * scale, 28.0 * scale);
    let space_1_x = start_x + space_width / 2.0;
    let space_2_x = start_x + space_width + space_gap + space_width / 2.0;

    // Helper to draw a camel at position
    let draw_camel = |painter: &egui::Painter, x: f32, y: f32, color: CamelColor| {
        let camel_rect = egui::Rect::from_center_size(egui::pos2(x, y), camel_size);
        let egui_color = camel_color_to_egui(color);
        let border = egui::Color32::from_rgb(
            (egui_color.r() as f32 * 0.5) as u8,
            (egui_color.g() as f32 * 0.5) as u8,
            (egui_color.b() as f32 * 0.5) as u8,
        );
        draw_camel_silhouette(painter, camel_rect, egui_color, border);
    };

    let base_y = track_y - (14.0 * scale);
    let stack_offset = 14.0 * scale;

    match phase {
        0 => {
            // Phase 0: Blue camel on space 1
            draw_camel(painter, space_1_x, base_y, CamelColor::Blue);

            painter.text(
                egui::pos2(rect.center().x, rect.min.y + 20.0),
                egui::Align2::CENTER_CENTER,
                "Blue camel is on space 1",
                egui::FontId::proportional(14.0),
                egui::Color32::LIGHT_GRAY,
            );
        }
        1 => {
            // Phase 1: Red lands on Blue
            draw_camel(painter, space_1_x, base_y, CamelColor::Blue);

            let red_y = if phase_progress < 0.5 {
                // Dropping from above
                let drop_progress = phase_progress * 2.0;
                let start_y = rect.min.y + 30.0;
                let end_y = base_y - stack_offset;
                start_y + (end_y - start_y) * ease_out_cubic(drop_progress)
            } else {
                base_y - stack_offset
            };
            draw_camel(painter, space_1_x, red_y, CamelColor::Red);

            painter.text(
                egui::pos2(rect.center().x, rect.min.y + 20.0),
                egui::Align2::CENTER_CENTER,
                "Red lands on Blue -> stacks ON TOP",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(255, 180, 100),
            );
        }
        2 => {
            // Phase 2: Green lands on stack
            draw_camel(painter, space_1_x, base_y, CamelColor::Blue);
            draw_camel(painter, space_1_x, base_y - stack_offset, CamelColor::Red);

            let green_y = if phase_progress < 0.5 {
                let drop_progress = phase_progress * 2.0;
                let start_y = rect.min.y + 30.0;
                let end_y = base_y - stack_offset * 2.0;
                start_y + (end_y - start_y) * ease_out_cubic(drop_progress)
            } else {
                base_y - stack_offset * 2.0
            };
            draw_camel(painter, space_1_x, green_y, CamelColor::Green);

            painter.text(
                egui::pos2(rect.center().x, rect.min.y + 20.0),
                egui::Align2::CENTER_CENTER,
                "Green lands -> joins the stack on top!",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(100, 255, 100),
            );
        }
        _ => {
            // Phase 3: Stack moves together
            let move_progress = ease_out_cubic(phase_progress);
            let current_x = space_1_x + (space_2_x - space_1_x) * move_progress;

            draw_camel(painter, current_x, base_y, CamelColor::Blue);
            draw_camel(painter, current_x, base_y - stack_offset, CamelColor::Red);
            draw_camel(painter, current_x, base_y - stack_offset * 2.0, CamelColor::Green);

            painter.text(
                egui::pos2(rect.center().x, rect.min.y + 20.0),
                egui::Align2::CENTER_CENTER,
                "Blue moves -> carries Red and Green!",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(100, 180, 255),
            );
        }
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

// ============================================================================
// Betting Section
// ============================================================================

fn draw_betting_section(ui: &mut egui::Ui) {
    ui.heading(egui::RichText::new("Betting").size(20.0).color(egui::Color32::WHITE));
    ui.add_space(12.0);

    // Leg Bets
    ui.heading(egui::RichText::new("Leg Bets").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.label(egui::RichText::new(
        "Predict which camel will be in 1st or 2nd place when the leg ends. \
        Take a tile from your chosen camel's stack - higher values go first!"
    ).size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.add_space(8.0);

    // Draw example leg bet cards
    ui.horizontal(|ui| {
        for (color, value) in [(CamelColor::Blue, 5), (CamelColor::Green, 3), (CamelColor::Red, 2)] {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 55.0), egui::Sense::hover());
            draw_mini_leg_bet_card(ui.painter(), rect, color, value);
            ui.add_space(4.0);
        }
        ui.label(egui::RichText::new("← Tile values: 5, 3, 2").size(12.0).color(egui::Color32::GRAY));
    });

    ui.add_space(16.0);

    // Race Bets
    ui.heading(egui::RichText::new("Race Bets").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.label(egui::RichText::new(
        "Predict the overall winner or loser of the entire race! \
        You have one card for each camel color - use them wisely."
    ).size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.add_space(8.0);

    ui.label(egui::RichText::new("• First correct bet: $8").size(14.0).color(egui::Color32::from_rgb(100, 255, 100)));
    ui.label(egui::RichText::new("• Second correct bet: $5").size(14.0).color(egui::Color32::from_rgb(150, 255, 150)));
    ui.label(egui::RichText::new("• Third: $3, Fourth: $2, Fifth: $1").size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.label(egui::RichText::new("• Wrong bet: -$1").size(14.0).color(egui::Color32::from_rgb(255, 150, 150)));

    ui.add_space(16.0);

    // Pyramid Tokens
    ui.heading(egui::RichText::new("Pyramid Tokens").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
        draw_pyramid_token_icon(ui.painter(), rect.center(), 24.0);
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Earn $1 every time you roll the pyramid die!")
            .size(14.0).color(egui::Color32::LIGHT_GRAY));
    });
}

// ============================================================================
// Desert Tiles Section
// ============================================================================

fn draw_desert_tiles_section(ui: &mut egui::Ui) {
    ui.heading(egui::RichText::new("Desert Tiles").size(20.0).color(egui::Color32::WHITE));
    ui.add_space(12.0);

    ui.label(egui::RichText::new(
        "Place your desert tile on the track to affect camel movement. \
        You earn $1 whenever ANY camel lands on your tile!"
    ).size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.add_space(16.0);

    ui.horizontal(|ui| {
        // Oasis card
        ui.vertical(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(80.0, 80.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 8.0, OASIS_GREEN);
            ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(40, 100, 40)), egui::epaint::StrokeKind::Outside);
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "+1",
                egui::FontId::proportional(28.0), egui::Color32::WHITE);

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Oasis").size(14.0).color(OASIS_GREEN).strong());
        });

        ui.add_space(16.0);

        ui.vertical(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new("Camel moves +1 extra space").size(14.0).color(egui::Color32::LIGHT_GRAY));
            ui.label(egui::RichText::new("Lands ON TOP of any stack").size(14.0).color(egui::Color32::from_rgb(100, 255, 100)));
        });
    });

    ui.add_space(16.0);

    ui.horizontal(|ui| {
        // Mirage card
        ui.vertical(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(80.0, 80.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 8.0, MIRAGE_ORANGE);
            ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(150, 100, 40)), egui::epaint::StrokeKind::Outside);
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "-1",
                egui::FontId::proportional(28.0), egui::Color32::WHITE);

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Mirage").size(14.0).color(MIRAGE_ORANGE).strong());
        });

        ui.add_space(16.0);

        ui.vertical(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new("Camel moves -1 space (backwards)").size(14.0).color(egui::Color32::LIGHT_GRAY));
            ui.label(egui::RichText::new("Lands UNDERNEATH any stack").size(14.0).color(egui::Color32::from_rgb(255, 150, 150)));
        });
    });

    ui.add_space(16.0);

    // Placement rules
    ui.heading(egui::RichText::new("Placement Rules").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.label(egui::RichText::new("• Cannot place on a space with camels")
        .size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.label(egui::RichText::new("• Cannot place adjacent to another desert tile")
        .size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.label(egui::RichText::new("• Cannot place on the starting space (0)")
        .size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.label(egui::RichText::new("• Your tile returns to you at the end of each leg")
        .size(14.0).color(egui::Color32::from_rgb(200, 200, 150)));
}

// ============================================================================
// Scoring Section
// ============================================================================

fn draw_scoring_section(ui: &mut egui::Ui) {
    ui.heading(egui::RichText::new("Scoring").size(20.0).color(egui::Color32::WHITE));
    ui.add_space(12.0);

    // Leg Scoring
    ui.heading(egui::RichText::new("Leg Scoring").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.label(egui::RichText::new("When 5 dice have been rolled (including crazy camels), the leg ends:")
        .size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.add_space(8.0);

    // Payout table
    egui::Grid::new("leg_scoring_table")
        .striped(true)
        .min_col_width(100.0)
        .show(ui, |ui| {
            // Header
            ui.label(egui::RichText::new("Your Bet").color(egui::Color32::WHITE).strong());
            ui.label(egui::RichText::new("Payout").color(egui::Color32::WHITE).strong());
            ui.end_row();

            ui.label(egui::RichText::new("1st Place").color(egui::Color32::LIGHT_GRAY));
            ui.label(egui::RichText::new("Tile Value (5/3/2)").color(egui::Color32::from_rgb(100, 255, 100)));
            ui.end_row();

            ui.label(egui::RichText::new("2nd Place").color(egui::Color32::LIGHT_GRAY));
            ui.label(egui::RichText::new("$1").color(egui::Color32::from_rgb(150, 255, 150)));
            ui.end_row();

            ui.label(egui::RichText::new("3rd or worse").color(egui::Color32::LIGHT_GRAY));
            ui.label(egui::RichText::new("-$1").color(egui::Color32::from_rgb(255, 150, 150)));
            ui.end_row();
        });

    ui.add_space(16.0);

    // Race Scoring
    ui.heading(egui::RichText::new("Race Scoring (Game End)").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.label(egui::RichText::new("Winner & Loser bets are revealed in order placed:")
        .size(14.0).color(egui::Color32::LIGHT_GRAY));
    ui.add_space(8.0);

    egui::Grid::new("race_scoring_table")
        .striped(true)
        .min_col_width(80.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Order").color(egui::Color32::WHITE).strong());
            ui.label(egui::RichText::new("Correct").color(egui::Color32::WHITE).strong());
            ui.label(egui::RichText::new("Wrong").color(egui::Color32::WHITE).strong());
            ui.end_row();

            for (order, payout) in [("1st", 8), ("2nd", 5), ("3rd", 3), ("4th", 2), ("5th+", 1)] {
                ui.label(egui::RichText::new(order).color(egui::Color32::LIGHT_GRAY));
                ui.label(egui::RichText::new(format!("${}", payout)).color(egui::Color32::from_rgb(100, 255, 100)));
                ui.label(egui::RichText::new("-$1").color(egui::Color32::from_rgb(255, 150, 150)));
                ui.end_row();
            }
        });

    ui.add_space(16.0);

    // Other income
    ui.heading(egui::RichText::new("Other Income").size(16.0).color(egui::Color32::WHITE));
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
        draw_pyramid_token_icon(ui.painter(), rect.center(), 20.0);
        ui.label(egui::RichText::new(" Pyramid token: +$1 each").size(14.0).color(egui::Color32::LIGHT_GRAY));
    });
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 3.0, OASIS_GREEN);
        ui.label(egui::RichText::new(" Desert tile landing: +$1 each").size(14.0).color(egui::Color32::LIGHT_GRAY));
    });
}

