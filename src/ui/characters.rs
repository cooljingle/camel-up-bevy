use bevy_egui::egui::{self, Color32, Pos2, Rect, Stroke};

/// Unique character identifiers for player avatars
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum CharacterId {
    #[default]
    DesertExplorer = 0,
    Merchant = 1,
    Princess = 2,
    Jockey = 3,
    Pharaoh = 4,
    Nomad = 5,
    Scholar = 6,
    FortuneTeller = 7,
}

impl CharacterId {
    /// Get character from index (0-7), wrapping if needed
    pub fn from_index(index: usize) -> Self {
        match index % 8 {
            0 => Self::DesertExplorer,
            1 => Self::Merchant,
            2 => Self::Princess,
            3 => Self::Jockey,
            4 => Self::Pharaoh,
            5 => Self::Nomad,
            6 => Self::Scholar,
            _ => Self::FortuneTeller,
        }
    }

    /// Get display name for the character
    pub fn name(&self) -> &'static str {
        match self {
            Self::DesertExplorer => "Explorer",
            Self::Merchant => "Merchant",
            Self::Princess => "Princess",
            Self::Jockey => "Jockey",
            Self::Pharaoh => "Pharaoh",
            Self::Nomad => "Nomad",
            Self::Scholar => "Scholar",
            Self::FortuneTeller => "Mystic",
        }
    }
}

/// Skin tone colors
const SKIN_LIGHT: Color32 = Color32::from_rgb(255, 220, 185);
const SKIN_TAN: Color32 = Color32::from_rgb(210, 170, 130);
const SKIN_MEDIUM: Color32 = Color32::from_rgb(180, 140, 100);
const SKIN_DARK: Color32 = Color32::from_rgb(140, 100, 70);

/// Draw a character avatar in the given rect
pub fn draw_avatar(painter: &egui::Painter, rect: Rect, character: CharacterId, border_color: Option<Color32>) {
    draw_avatar_with_expression(painter, rect, character, border_color, false)
}

/// Draw a character avatar with optional happy expression (for winners)
pub fn draw_avatar_with_expression(painter: &egui::Painter, rect: Rect, character: CharacterId, border_color: Option<Color32>, happy: bool) {
    let center = rect.center();
    let size = rect.width().min(rect.height());
    let radius = size * 0.45;

    // Draw border/background if specified
    if let Some(border) = border_color {
        painter.circle_filled(center, radius + 3.0, border);
    }

    // Draw character based on type
    match character {
        CharacterId::DesertExplorer => draw_explorer(painter, center, radius, happy),
        CharacterId::Merchant => draw_merchant(painter, center, radius, happy),
        CharacterId::Princess => draw_princess(painter, center, radius, happy),
        CharacterId::Jockey => draw_jockey(painter, center, radius, happy),
        CharacterId::Pharaoh => draw_pharaoh(painter, center, radius, happy),
        CharacterId::Nomad => draw_nomad(painter, center, radius, happy),
        CharacterId::Scholar => draw_scholar(painter, center, radius, happy),
        CharacterId::FortuneTeller => draw_fortune_teller(painter, center, radius, happy),
    }
}

/// Desert Explorer - Safari hat, tan skin, adventurous
fn draw_explorer(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_TAN;
    let hat_color = Color32::from_rgb(160, 130, 80); // Khaki
    let hat_band = Color32::from_rgb(100, 70, 40);

    // Face
    painter.circle_filled(center, radius * 0.85, skin);

    // Safari hat (wide brim)
    let hat_top = center.y - radius * 0.3;
    painter.rect_filled(
        Rect::from_center_size(
            Pos2::new(center.x, hat_top - radius * 0.15),
            egui::vec2(radius * 1.8, radius * 0.4),
        ),
        radius * 0.1,
        hat_color,
    );
    // Hat crown
    painter.rect_filled(
        Rect::from_center_size(
            Pos2::new(center.x, hat_top - radius * 0.45),
            egui::vec2(radius * 1.0, radius * 0.4),
        ),
        radius * 0.15,
        hat_color,
    );
    // Hat band
    painter.rect_filled(
        Rect::from_center_size(
            Pos2::new(center.x, hat_top - radius * 0.25),
            egui::vec2(radius * 1.0, radius * 0.1),
        ),
        0.0,
        hat_band,
    );

    // Eyes
    draw_eyes(painter, center, radius, Color32::from_rgb(80, 60, 40));

    // Smile - big for happy, normal otherwise
    if happy {
        draw_big_smile(painter, center, radius);
    } else {
        draw_smile(painter, center, radius);
    }
}

/// Merchant - Turban, beard, shrewd expression
fn draw_merchant(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_MEDIUM;
    let turban_color = Color32::from_rgb(180, 50, 50); // Red turban
    let beard_color = Color32::from_rgb(40, 30, 20);

    // Face
    painter.circle_filled(center, radius * 0.85, skin);

    // Beard
    let beard_center = Pos2::new(center.x, center.y + radius * 0.35);
    painter.circle_filled(beard_center, radius * 0.55, beard_color);
    // Beard is behind face, redraw lower face
    painter.circle_filled(
        Pos2::new(center.x, center.y - radius * 0.1),
        radius * 0.7,
        skin,
    );

    // Turban
    let turban_y = center.y - radius * 0.45;
    painter.circle_filled(Pos2::new(center.x, turban_y), radius * 0.65, turban_color);
    // Turban gem
    painter.circle_filled(
        Pos2::new(center.x, turban_y + radius * 0.2),
        radius * 0.12,
        Color32::from_rgb(50, 200, 100), // Green gem
    );

    // Eyes (slightly narrowed, shrewd)
    let eye_y = center.y - radius * 0.05;
    let eye_offset = radius * 0.25;
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.12, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.12, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.06, Color32::from_rgb(60, 40, 20));
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.06, Color32::from_rgb(60, 40, 20));

    // Smile
    if happy {
        draw_big_smile(painter, center, radius);
    } else {
        draw_smile(painter, center, radius);
    }
}

/// Princess - Tiara, elegant, long eyelashes
fn draw_princess(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_LIGHT;
    let hair_color = Color32::from_rgb(60, 30, 10); // Dark brown
    let tiara_color = Color32::from_rgb(255, 215, 0); // Gold

    // Hair background
    painter.circle_filled(center, radius * 0.95, hair_color);

    // Face
    painter.circle_filled(
        Pos2::new(center.x, center.y + radius * 0.1),
        radius * 0.75,
        skin,
    );

    // Tiara
    let tiara_y = center.y - radius * 0.55;
    // Base
    painter.rect_filled(
        Rect::from_center_size(
            Pos2::new(center.x, tiara_y + radius * 0.1),
            egui::vec2(radius * 1.2, radius * 0.15),
        ),
        radius * 0.05,
        tiara_color,
    );
    // Center peak
    let peak_points = [
        Pos2::new(center.x, tiara_y - radius * 0.2),
        Pos2::new(center.x - radius * 0.15, tiara_y + radius * 0.1),
        Pos2::new(center.x + radius * 0.15, tiara_y + radius * 0.1),
    ];
    painter.add(egui::Shape::convex_polygon(peak_points.to_vec(), tiara_color, Stroke::NONE));
    // Side peaks
    for offset in [-0.35, 0.35] {
        let peak = [
            Pos2::new(center.x + radius * offset, tiara_y - radius * 0.1),
            Pos2::new(center.x + radius * (offset - 0.1), tiara_y + radius * 0.1),
            Pos2::new(center.x + radius * (offset + 0.1), tiara_y + radius * 0.1),
        ];
        painter.add(egui::Shape::convex_polygon(peak.to_vec(), tiara_color, Stroke::NONE));
    }
    // Gem in center
    painter.circle_filled(
        Pos2::new(center.x, tiara_y),
        radius * 0.08,
        Color32::from_rgb(200, 50, 100), // Ruby
    );

    // Eyes with lashes
    let eye_y = center.y + radius * 0.05;
    let eye_offset = radius * 0.22;
    // Eyes
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.13, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.13, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.07, Color32::from_rgb(50, 100, 150));
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.07, Color32::from_rgb(50, 100, 150));
    // Eyelashes (simple lines above eyes)
    for i in -1..=1 {
        let lash_x_left = center.x - eye_offset + (i as f32) * radius * 0.08;
        let lash_x_right = center.x + eye_offset + (i as f32) * radius * 0.08;
        painter.line_segment(
            [Pos2::new(lash_x_left, eye_y - radius * 0.12), Pos2::new(lash_x_left, eye_y - radius * 0.2)],
            Stroke::new(1.5, Color32::BLACK),
        );
        painter.line_segment(
            [Pos2::new(lash_x_right, eye_y - radius * 0.12), Pos2::new(lash_x_right, eye_y - radius * 0.2)],
            Stroke::new(1.5, Color32::BLACK),
        );
    }

    // Smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.1);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.8);
    } else {
        draw_smile(painter, smile_center, radius * 0.8);
    }
}

/// Jockey - Racing helmet, goggles pushed up
fn draw_jockey(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_LIGHT;
    let helmet_color = Color32::from_rgb(200, 30, 30); // Red helmet
    let goggle_color = Color32::from_rgb(50, 50, 50);

    // Face
    painter.circle_filled(center, radius * 0.8, skin);

    // Helmet
    let helmet_y = center.y - radius * 0.2;
    painter.circle_filled(Pos2::new(center.x, helmet_y), radius * 0.85, helmet_color);
    // Helmet stripe
    painter.rect_filled(
        Rect::from_center_size(
            Pos2::new(center.x, helmet_y),
            egui::vec2(radius * 0.2, radius * 1.4),
        ),
        0.0,
        Color32::WHITE,
    );
    // Re-draw face area
    painter.circle_filled(
        Pos2::new(center.x, center.y + radius * 0.15),
        radius * 0.6,
        skin,
    );

    // Goggles on forehead
    let goggle_y = center.y - radius * 0.25;
    // Strap
    painter.rect_filled(
        Rect::from_center_size(
            Pos2::new(center.x, goggle_y),
            egui::vec2(radius * 1.4, radius * 0.12),
        ),
        0.0,
        goggle_color,
    );
    // Lenses
    painter.circle_filled(Pos2::new(center.x - radius * 0.3, goggle_y), radius * 0.18, Color32::from_rgb(180, 200, 220));
    painter.circle_filled(Pos2::new(center.x + radius * 0.3, goggle_y), radius * 0.18, Color32::from_rgb(180, 200, 220));
    painter.circle_stroke(Pos2::new(center.x - radius * 0.3, goggle_y), radius * 0.18, Stroke::new(2.0, goggle_color));
    painter.circle_stroke(Pos2::new(center.x + radius * 0.3, goggle_y), radius * 0.18, Stroke::new(2.0, goggle_color));

    // Eyes
    draw_eyes(painter, Pos2::new(center.x, center.y + radius * 0.1), radius * 0.8, Color32::from_rgb(60, 120, 60));

    // Smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.1);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.7);
    } else {
        draw_smile(painter, smile_center, radius * 0.7);
    }
}

/// Pharaoh - Egyptian headdress, regal bearing
fn draw_pharaoh(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_TAN;
    let headdress_color = Color32::from_rgb(30, 80, 160); // Royal blue
    let gold = Color32::from_rgb(255, 200, 50);

    // Face
    painter.circle_filled(center, radius * 0.7, skin);

    // Nemes headdress (striped)
    // Main headdress shape
    let head_top = center.y - radius * 0.6;
    painter.rect_filled(
        Rect::from_center_size(
            Pos2::new(center.x, head_top),
            egui::vec2(radius * 1.6, radius * 0.8),
        ),
        radius * 0.1,
        headdress_color,
    );
    // Side flaps
    let flap_points_left = [
        Pos2::new(center.x - radius * 0.7, center.y - radius * 0.2),
        Pos2::new(center.x - radius * 0.9, center.y + radius * 0.7),
        Pos2::new(center.x - radius * 0.4, center.y + radius * 0.5),
    ];
    let flap_points_right = [
        Pos2::new(center.x + radius * 0.7, center.y - radius * 0.2),
        Pos2::new(center.x + radius * 0.9, center.y + radius * 0.7),
        Pos2::new(center.x + radius * 0.4, center.y + radius * 0.5),
    ];
    painter.add(egui::Shape::convex_polygon(flap_points_left.to_vec(), headdress_color, Stroke::NONE));
    painter.add(egui::Shape::convex_polygon(flap_points_right.to_vec(), headdress_color, Stroke::NONE));

    // Gold headband
    painter.rect_filled(
        Rect::from_center_size(
            Pos2::new(center.x, center.y - radius * 0.35),
            egui::vec2(radius * 1.5, radius * 0.12),
        ),
        0.0,
        gold,
    );

    // Uraeus (cobra) symbol
    painter.circle_filled(
        Pos2::new(center.x, center.y - radius * 0.5),
        radius * 0.12,
        gold,
    );

    // Re-draw face
    painter.circle_filled(
        Pos2::new(center.x, center.y + radius * 0.05),
        radius * 0.55,
        skin,
    );

    // Kohl-lined eyes
    let eye_y = center.y + radius * 0.0;
    let eye_offset = radius * 0.2;
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.12, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.12, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.06, Color32::from_rgb(50, 40, 30));
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.06, Color32::from_rgb(50, 40, 30));
    // Eyeliner
    painter.line_segment(
        [Pos2::new(center.x - eye_offset - radius * 0.1, eye_y), Pos2::new(center.x - eye_offset - radius * 0.25, eye_y + radius * 0.1)],
        Stroke::new(2.0, Color32::BLACK),
    );
    painter.line_segment(
        [Pos2::new(center.x + eye_offset + radius * 0.1, eye_y), Pos2::new(center.x + eye_offset + radius * 0.25, eye_y + radius * 0.1)],
        Stroke::new(2.0, Color32::BLACK),
    );

    // Smile - happy or regal neutral
    if happy {
        draw_big_smile(painter, center, radius * 0.8);
    } else {
        draw_smile(painter, center, radius * 0.8);
    }
}

/// Nomad - Head scarf, weathered look
fn draw_nomad(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_DARK;
    let scarf_color = Color32::from_rgb(180, 160, 120); // Sandy beige
    let wrap_color = Color32::from_rgb(140, 120, 90);

    // Face (partially covered)
    painter.circle_filled(center, radius * 0.85, skin);

    // Head scarf/keffiyeh
    // Top of head
    painter.circle_filled(
        Pos2::new(center.x, center.y - radius * 0.3),
        radius * 0.75,
        scarf_color,
    );
    // Draping sides
    let left_drape = [
        Pos2::new(center.x - radius * 0.6, center.y - radius * 0.3),
        Pos2::new(center.x - radius * 0.8, center.y + radius * 0.6),
        Pos2::new(center.x - radius * 0.2, center.y + radius * 0.4),
    ];
    let right_drape = [
        Pos2::new(center.x + radius * 0.6, center.y - radius * 0.3),
        Pos2::new(center.x + radius * 0.8, center.y + radius * 0.6),
        Pos2::new(center.x + radius * 0.2, center.y + radius * 0.4),
    ];
    painter.add(egui::Shape::convex_polygon(left_drape.to_vec(), scarf_color, Stroke::NONE));
    painter.add(egui::Shape::convex_polygon(right_drape.to_vec(), scarf_color, Stroke::NONE));

    // Face wrap (covers lower face) - only if not happy
    if !happy {
        painter.rect_filled(
            Rect::from_center_size(
                Pos2::new(center.x, center.y + radius * 0.35),
                egui::vec2(radius * 1.2, radius * 0.5),
            ),
            0.0,
            wrap_color,
        );

        // Re-draw visible face area (eyes only)
        painter.rect_filled(
            Rect::from_center_size(
                Pos2::new(center.x, center.y - radius * 0.05),
                egui::vec2(radius * 1.0, radius * 0.4),
            ),
            0.0,
            skin,
        );
    } else {
        // When happy, show more face with wrap pulled down
        painter.rect_filled(
            Rect::from_center_size(
                Pos2::new(center.x, center.y + radius * 0.55),
                egui::vec2(radius * 1.2, radius * 0.4),
            ),
            0.0,
            wrap_color,
        );

        // Draw more of the face
        painter.circle_filled(
            Pos2::new(center.x, center.y + radius * 0.1),
            radius * 0.5,
            skin,
        );
    }

    // Intense eyes
    let eye_y = center.y - radius * 0.05;
    let eye_offset = radius * 0.25;
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.12, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.12, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.07, Color32::from_rgb(40, 30, 20));
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.07, Color32::from_rgb(40, 30, 20));
    // Eyebrows (thick, weathered)
    painter.line_segment(
        [Pos2::new(center.x - eye_offset - radius * 0.12, eye_y - radius * 0.15), Pos2::new(center.x - eye_offset + radius * 0.12, eye_y - radius * 0.18)],
        Stroke::new(3.0, Color32::from_rgb(30, 20, 10)),
    );
    painter.line_segment(
        [Pos2::new(center.x + eye_offset - radius * 0.12, eye_y - radius * 0.18), Pos2::new(center.x + eye_offset + radius * 0.12, eye_y - radius * 0.15)],
        Stroke::new(3.0, Color32::from_rgb(30, 20, 10)),
    );

    // Smile when happy (visible since wrap is pulled down)
    if happy {
        draw_big_smile(painter, Pos2::new(center.x, center.y + radius * 0.1), radius * 0.7);
    }
}

/// Scholar - Glasses, book, thoughtful expression
fn draw_scholar(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_LIGHT;
    let hair_color = Color32::from_rgb(80, 60, 40);
    let glasses_color = Color32::from_rgb(50, 50, 50);

    // Hair
    painter.circle_filled(
        Pos2::new(center.x, center.y - radius * 0.2),
        radius * 0.8,
        hair_color,
    );

    // Face
    painter.circle_filled(
        Pos2::new(center.x, center.y + radius * 0.1),
        radius * 0.7,
        skin,
    );

    // Glasses
    let glass_y = center.y + radius * 0.0;
    let glass_offset = radius * 0.28;
    // Frames
    painter.circle_stroke(Pos2::new(center.x - glass_offset, glass_y), radius * 0.2, Stroke::new(2.5, glasses_color));
    painter.circle_stroke(Pos2::new(center.x + glass_offset, glass_y), radius * 0.2, Stroke::new(2.5, glasses_color));
    // Bridge
    painter.line_segment(
        [Pos2::new(center.x - glass_offset + radius * 0.2, glass_y), Pos2::new(center.x + glass_offset - radius * 0.2, glass_y)],
        Stroke::new(2.5, glasses_color),
    );
    // Temples
    painter.line_segment(
        [Pos2::new(center.x - glass_offset - radius * 0.2, glass_y), Pos2::new(center.x - radius * 0.7, glass_y - radius * 0.1)],
        Stroke::new(2.5, glasses_color),
    );
    painter.line_segment(
        [Pos2::new(center.x + glass_offset + radius * 0.2, glass_y), Pos2::new(center.x + radius * 0.7, glass_y - radius * 0.1)],
        Stroke::new(2.5, glasses_color),
    );

    // Eyes behind glasses
    painter.circle_filled(Pos2::new(center.x - glass_offset, glass_y), radius * 0.08, Color32::from_rgb(60, 80, 100));
    painter.circle_filled(Pos2::new(center.x + glass_offset, glass_y), radius * 0.08, Color32::from_rgb(60, 80, 100));

    // Smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.1);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.7);
    } else {
        draw_smile(painter, smile_center, radius * 0.7);
    }
}

/// Fortune Teller - Mystical veil, jewelry
fn draw_fortune_teller(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_MEDIUM;
    let veil_color = Color32::from_rgb(80, 20, 120); // Deep purple
    let gold = Color32::from_rgb(255, 200, 50);

    // Mystical veil/hood
    painter.circle_filled(center, radius * 0.95, veil_color);

    // Face opening
    painter.circle_filled(
        Pos2::new(center.x, center.y + radius * 0.1),
        radius * 0.65,
        skin,
    );

    // Headpiece with gems
    let headpiece_y = center.y - radius * 0.35;
    painter.rect_filled(
        Rect::from_center_size(
            Pos2::new(center.x, headpiece_y),
            egui::vec2(radius * 1.3, radius * 0.15),
        ),
        0.0,
        gold,
    );
    // Center gem (third eye)
    painter.circle_filled(
        Pos2::new(center.x, headpiece_y),
        radius * 0.1,
        Color32::from_rgb(100, 200, 255), // Mystic blue
    );
    // Side gems
    painter.circle_filled(Pos2::new(center.x - radius * 0.35, headpiece_y), radius * 0.06, Color32::from_rgb(255, 100, 100));
    painter.circle_filled(Pos2::new(center.x + radius * 0.35, headpiece_y), radius * 0.06, Color32::from_rgb(100, 255, 100));

    // Mysterious eyes
    let eye_y = center.y + radius * 0.05;
    let eye_offset = radius * 0.22;
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.14, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.14, Color32::WHITE);
    // Purple mystical iris
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.08, Color32::from_rgb(120, 80, 180));
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.08, Color32::from_rgb(120, 80, 180));
    // Tiny reflection
    painter.circle_filled(Pos2::new(center.x - eye_offset + radius * 0.03, eye_y - radius * 0.03), radius * 0.02, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x + eye_offset + radius * 0.03, eye_y - radius * 0.03), radius * 0.02, Color32::WHITE);

    // Earrings
    painter.circle_filled(Pos2::new(center.x - radius * 0.55, center.y + radius * 0.15), radius * 0.08, gold);
    painter.circle_filled(Pos2::new(center.x + radius * 0.55, center.y + radius * 0.15), radius * 0.08, gold);

    // Smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.1);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.8);
    } else {
        draw_smile(painter, smile_center, radius * 0.8);
    }
}

/// Helper: Draw standard eyes
fn draw_eyes(painter: &egui::Painter, center: Pos2, radius: f32, iris_color: Color32) {
    let eye_y = center.y - radius * 0.05;
    let eye_offset = radius * 0.25;

    // Eye whites
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.13, Color32::WHITE);
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.13, Color32::WHITE);

    // Irises
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.07, iris_color);
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.07, iris_color);

    // Pupils
    painter.circle_filled(Pos2::new(center.x - eye_offset, eye_y), radius * 0.03, Color32::BLACK);
    painter.circle_filled(Pos2::new(center.x + eye_offset, eye_y), radius * 0.03, Color32::BLACK);
}

/// Helper: Draw a simple smile (curves upward = happy)
fn draw_smile(painter: &egui::Painter, center: Pos2, radius: f32) {
    let smile_y = center.y + radius * 0.25;
    let smile_width = radius * 0.25;

    // Simple curved smile using line segments - curves DOWN from edges to make upward smile
    let segments = 5;
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;

        let x1 = center.x - smile_width + t1 * smile_width * 2.0;
        let x2 = center.x - smile_width + t2 * smile_width * 2.0;

        // Parabolic curve - negative to curve upward (happy smile)
        let curve1 = -(t1 - 0.5).powi(2) * 0.4 + 0.1;
        let curve2 = -(t2 - 0.5).powi(2) * 0.4 + 0.1;

        let y1 = smile_y + curve1 * radius;
        let y2 = smile_y + curve2 * radius;

        painter.line_segment(
            [Pos2::new(x1, y1), Pos2::new(x2, y2)],
            Stroke::new(2.0, Color32::from_rgb(180, 100, 90)),
        );
    }
}

/// Helper: Draw a big happy smile (for winners)
fn draw_big_smile(painter: &egui::Painter, center: Pos2, radius: f32) {
    let smile_y = center.y + radius * 0.22;
    let smile_width = radius * 0.35;

    // Wide happy grin - deeper curve
    let segments = 8;
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;

        let x1 = center.x - smile_width + t1 * smile_width * 2.0;
        let x2 = center.x - smile_width + t2 * smile_width * 2.0;

        // Deeper parabolic curve for big smile
        let curve1 = -(t1 - 0.5).powi(2) * 0.6 + 0.15;
        let curve2 = -(t2 - 0.5).powi(2) * 0.6 + 0.15;

        let y1 = smile_y + curve1 * radius;
        let y2 = smile_y + curve2 * radius;

        painter.line_segment(
            [Pos2::new(x1, y1), Pos2::new(x2, y2)],
            Stroke::new(2.5, Color32::from_rgb(180, 100, 90)),
        );
    }

    // Add teeth/open mouth effect - fill the smile area
    let mouth_top = smile_y - radius * 0.05;
    let mouth_bottom = smile_y + radius * 0.12;
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(center.x - smile_width * 0.7, mouth_top),
            Pos2::new(center.x + smile_width * 0.7, mouth_bottom),
        ),
        radius * 0.05,
        Color32::from_rgb(255, 240, 240), // Teeth white
    );
}
