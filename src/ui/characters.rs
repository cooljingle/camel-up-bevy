use bevy_egui::egui::{self, Color32, Pos2, Rect, Stroke};

// ============================================================================
// Color manipulation helpers
// ============================================================================

/// Darken a color by a factor (0.0 = no change, 1.0 = black)
fn darken(color: Color32, factor: f32) -> Color32 {
    Color32::from_rgb(
        (color.r() as f32 * (1.0 - factor)) as u8,
        (color.g() as f32 * (1.0 - factor)) as u8,
        (color.b() as f32 * (1.0 - factor)) as u8,
    )
}

/// Lighten a color by a factor (0.0 = no change, 1.0 = white)
fn lighten(color: Color32, factor: f32) -> Color32 {
    Color32::from_rgb(
        (color.r() as f32 + (255.0 - color.r() as f32) * factor) as u8,
        (color.g() as f32 + (255.0 - color.g() as f32) * factor) as u8,
        (color.b() as f32 + (255.0 - color.b() as f32) * factor) as u8,
    )
}

// ============================================================================
// Layered rectangle drawing (camel-style 4-layer system)
// ============================================================================

/// Draw a rectangle with 4 layers: shadow, border, main, highlight
/// This matches the visual style of the camel sprites
fn draw_layered_rect(
    painter: &egui::Painter,
    rect: Rect,
    rounding: f32,
    base_color: Color32,
) {
    let shadow_offset = rect.width() * 0.05;
    let border_inset = rect.width() * 0.03;
    let highlight_inset = rect.width() * 0.1;

    // Derive colors from base
    let shadow_color = darken(base_color, 0.5);
    let border_color = darken(base_color, 0.25);
    let highlight_color = lighten(base_color, 0.25);

    // Layer 1: Shadow (offset down-right)
    let shadow_rect = rect.translate(egui::vec2(shadow_offset, shadow_offset));
    painter.rect_filled(shadow_rect, rounding, shadow_color);

    // Layer 2: Border
    painter.rect_filled(rect, rounding, border_color);

    // Layer 3: Main color
    let main_rect = rect.shrink(border_inset);
    painter.rect_filled(main_rect, rounding * 0.9, base_color);

    // Layer 4: Highlight (top-left corner)
    let highlight_rect = Rect::from_min_size(
        rect.min + egui::vec2(highlight_inset, highlight_inset),
        egui::vec2(rect.width() * 0.25, rect.height() * 0.12),
    );
    painter.rect_filled(highlight_rect, rounding * 0.4, highlight_color);
}

/// Draw a simple layered rectangle without highlight (for smaller elements)
fn draw_simple_layered_rect(
    painter: &egui::Painter,
    rect: Rect,
    rounding: f32,
    base_color: Color32,
) {
    let shadow_offset = rect.width() * 0.08;
    let border_inset = rect.width() * 0.06;

    let shadow_color = darken(base_color, 0.4);
    let border_color = darken(base_color, 0.2);

    // Shadow
    let shadow_rect = rect.translate(egui::vec2(shadow_offset, shadow_offset));
    painter.rect_filled(shadow_rect, rounding, shadow_color);

    // Border
    painter.rect_filled(rect, rounding, border_color);

    // Main
    let main_rect = rect.shrink(border_inset);
    painter.rect_filled(main_rect, rounding * 0.8, base_color);
}

// ============================================================================
// Rectangular eye drawing
// ============================================================================

/// Draw rectangular eyes in the camel style
fn draw_rect_eyes(painter: &egui::Painter, center: Pos2, radius: f32, iris_color: Color32) {
    let eye_y = center.y - radius * 0.05;
    let eye_offset = radius * 0.25;
    let eye_width = radius * 0.24;
    let eye_height = radius * 0.18;
    let rounding = radius * 0.06;

    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;

        // Eye white - horizontal rounded rectangle
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(eye_width, eye_height),
        );
        painter.rect_filled(eye_rect, rounding, Color32::WHITE);

        // Iris - smaller rounded rectangle
        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.12, radius * 0.14),
        );
        painter.rect_filled(iris_rect, rounding * 0.5, iris_color);

        // Pupil - tiny rectangle
        let pupil_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.05, radius * 0.07),
        );
        painter.rect_filled(pupil_rect, rounding * 0.2, Color32::BLACK);
    }
}

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

    /// Get a list of thematic names for this character type
    pub fn thematic_names(&self) -> &'static [&'static str] {
        match self {
            Self::DesertExplorer => &["Sandy", "Indy", "Rex", "Dusty", "Marco", "Sahara", "Dunes", "Compass"],
            Self::Merchant => &["Hakim", "Jabari", "Omar", "Rashid", "Tariq", "Bazaar", "Silk", "Spice"],
            Self::Princess => &["Nefertiti", "Cleo", "Isis", "Amira", "Jasmine", "Lotus", "Pearl", "Sapphire"],
            Self::Jockey => &["Flash", "Speedy", "Blaze", "Dash", "Swift", "Bolt", "Thunder", "Rocket"],
            Self::Pharaoh => &["Ramses", "Tut", "Khufu", "Osiris", "Ra", "Anubis", "Horus", "Sphinx"],
            Self::Nomad => &["Bedouin", "Sahir", "Zephyr", "Dune", "Sirocco", "Mirage", "Wanderer", "Breeze"],
            Self::Scholar => &["Thoth", "Scribe", "Ptolemy", "Archie", "Sage", "Newton", "Wisdom", "Scroll"],
            Self::FortuneTeller => &["Oracle", "Sybil", "Cass", "Pythia", "Esme", "Zara", "Mystic", "Tarot"],
        }
    }

    /// Pick a random thematic name for this character
    pub fn random_name(&self) -> String {
        use rand::seq::SliceRandom;
        let names = self.thematic_names();
        let mut rng = rand::thread_rng();
        names.choose(&mut rng).unwrap_or(&"Player").to_string()
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

    // Draw border/background if specified (rounded rectangle to match camel style)
    if let Some(border) = border_color {
        let border_rect = Rect::from_center_size(
            center,
            egui::vec2(radius * 2.0 + 6.0, radius * 2.0 + 6.0),
        );
        painter.rect_filled(border_rect, radius * 0.4, border);
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

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.1),
        egui::vec2(radius * 1.5, radius * 1.6),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Safari hat (wide brim) - layered style
    let hat_top = center.y - radius * 0.3;
    let brim_rect = Rect::from_center_size(
        Pos2::new(center.x, hat_top - radius * 0.15),
        egui::vec2(radius * 1.8, radius * 0.4),
    );
    draw_simple_layered_rect(painter, brim_rect, radius * 0.1, hat_color);

    // Hat crown - layered style
    let crown_rect = Rect::from_center_size(
        Pos2::new(center.x, hat_top - radius * 0.45),
        egui::vec2(radius * 1.0, radius * 0.4),
    );
    draw_simple_layered_rect(painter, crown_rect, radius * 0.15, hat_color);

    // Hat band
    let band_rect = Rect::from_center_size(
        Pos2::new(center.x, hat_top - radius * 0.25),
        egui::vec2(radius * 1.0, radius * 0.1),
    );
    painter.rect_filled(band_rect, radius * 0.02, hat_band);

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

    // Beard - rounded rectangle at bottom
    let beard_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.45),
        egui::vec2(radius * 1.3, radius * 0.9),
    );
    draw_simple_layered_rect(painter, beard_rect, radius * 0.25, beard_color);

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.05),
        egui::vec2(radius * 1.4, radius * 1.3),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Turban - rounded rectangle (wider at top)
    let turban_y = center.y - radius * 0.45;
    let turban_rect = Rect::from_center_size(
        Pos2::new(center.x, turban_y),
        egui::vec2(radius * 1.4, radius * 0.7),
    );
    draw_layered_rect(painter, turban_rect, radius * 0.2, turban_color);

    // Turban gem - small rounded square
    let gem_rect = Rect::from_center_size(
        Pos2::new(center.x, turban_y + radius * 0.2),
        egui::vec2(radius * 0.2, radius * 0.2),
    );
    painter.rect_filled(gem_rect, radius * 0.05, Color32::from_rgb(50, 200, 100));

    // Eyes (slightly narrowed, shrewd) - rectangular style
    let eye_y = center.y - radius * 0.05;
    let eye_offset = radius * 0.25;
    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;
        // Narrower eyes for shrewd look
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.2, radius * 0.12),
        );
        painter.rect_filled(eye_rect, radius * 0.04, Color32::WHITE);
        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.1, radius * 0.1),
        );
        painter.rect_filled(iris_rect, radius * 0.03, Color32::from_rgb(60, 40, 20));
    }

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

    // Hair background - rounded rectangle
    let hair_rect = Rect::from_center_size(
        center,
        egui::vec2(radius * 1.9, radius * 1.9),
    );
    draw_simple_layered_rect(painter, hair_rect, radius * 0.4, hair_color);

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.15),
        egui::vec2(radius * 1.35, radius * 1.4),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Tiara
    let tiara_y = center.y - radius * 0.55;
    // Base - layered rectangle
    let tiara_base_rect = Rect::from_center_size(
        Pos2::new(center.x, tiara_y + radius * 0.1),
        egui::vec2(radius * 1.2, radius * 0.15),
    );
    draw_simple_layered_rect(painter, tiara_base_rect, radius * 0.05, tiara_color);

    // Center peak (keep as polygon for triangular shape)
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
    // Gem in center - rounded square
    let gem_rect = Rect::from_center_size(
        Pos2::new(center.x, tiara_y),
        egui::vec2(radius * 0.14, radius * 0.14),
    );
    painter.rect_filled(gem_rect, radius * 0.03, Color32::from_rgb(200, 50, 100));

    // Eyes with lashes - rectangular style
    let eye_y = center.y + radius * 0.05;
    let eye_offset = radius * 0.22;
    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;
        // Eye white
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.22, radius * 0.18),
        );
        painter.rect_filled(eye_rect, radius * 0.06, Color32::WHITE);
        // Iris
        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.12, radius * 0.14),
        );
        painter.rect_filled(iris_rect, radius * 0.04, Color32::from_rgb(50, 100, 150));
    }
    // Eyelashes (keep as line segments for detail)
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
    let lens_color = Color32::from_rgb(180, 200, 220);

    // Helmet - layered rounded rectangle
    let helmet_y = center.y - radius * 0.2;
    let helmet_rect = Rect::from_center_size(
        Pos2::new(center.x, helmet_y),
        egui::vec2(radius * 1.7, radius * 1.5),
    );
    draw_layered_rect(painter, helmet_rect, radius * 0.4, helmet_color);

    // Helmet stripe
    let stripe_rect = Rect::from_center_size(
        Pos2::new(center.x, helmet_y),
        egui::vec2(radius * 0.2, radius * 1.4),
    );
    painter.rect_filled(stripe_rect, radius * 0.05, Color32::WHITE);

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.2),
        egui::vec2(radius * 1.2, radius * 1.1),
    );
    draw_layered_rect(painter, face_rect, radius * 0.3, skin);

    // Goggles on forehead
    let goggle_y = center.y - radius * 0.25;
    // Strap - rounded rectangle
    let strap_rect = Rect::from_center_size(
        Pos2::new(center.x, goggle_y),
        egui::vec2(radius * 1.4, radius * 0.12),
    );
    painter.rect_filled(strap_rect, radius * 0.03, goggle_color);

    // Goggle lenses - rounded squares instead of circles
    for &x_mult in &[-1.0, 1.0] {
        let lens_x = center.x + radius * 0.3 * x_mult;
        // Lens fill
        let lens_rect = Rect::from_center_size(
            Pos2::new(lens_x, goggle_y),
            egui::vec2(radius * 0.32, radius * 0.28),
        );
        painter.rect_filled(lens_rect, radius * 0.08, lens_color);
        // Lens border
        painter.rect_stroke(lens_rect, radius * 0.08, Stroke::new(2.0, goggle_color), egui::epaint::StrokeKind::Outside);
    }

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

    // Nemes headdress (striped) - main headdress shape layered
    let head_top = center.y - radius * 0.6;
    let headdress_rect = Rect::from_center_size(
        Pos2::new(center.x, head_top),
        egui::vec2(radius * 1.6, radius * 0.8),
    );
    draw_layered_rect(painter, headdress_rect, radius * 0.1, headdress_color);

    // Side flaps (keep as polygons for triangular shape)
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

    // Gold headband - simple layered rectangle
    let headband_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - radius * 0.35),
        egui::vec2(radius * 1.5, radius * 0.12),
    );
    draw_simple_layered_rect(painter, headband_rect, radius * 0.03, gold);

    // Uraeus (cobra) symbol - rounded square instead of circle
    let uraeus_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - radius * 0.5),
        egui::vec2(radius * 0.2, radius * 0.2),
    );
    painter.rect_filled(uraeus_rect, radius * 0.05, gold);

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.1),
        egui::vec2(radius * 1.1, radius * 1.2),
    );
    draw_layered_rect(painter, face_rect, radius * 0.3, skin);

    // Kohl-lined eyes - rectangular style
    let eye_y = center.y + radius * 0.0;
    let eye_offset = radius * 0.2;
    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;
        // Eye white
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.2, radius * 0.16),
        );
        painter.rect_filled(eye_rect, radius * 0.05, Color32::WHITE);
        // Iris
        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.1, radius * 0.12),
        );
        painter.rect_filled(iris_rect, radius * 0.03, Color32::from_rgb(50, 40, 30));
    }
    // Eyeliner (keep as line segments for detail)
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

    // Head scarf/keffiyeh - rounded rectangle at top
    let scarf_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - radius * 0.3),
        egui::vec2(radius * 1.5, radius * 1.2),
    );
    draw_layered_rect(painter, scarf_rect, radius * 0.35, scarf_color);

    // Draping sides (keep as polygons for triangular shape)
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
        let wrap_rect = Rect::from_center_size(
            Pos2::new(center.x, center.y + radius * 0.35),
            egui::vec2(radius * 1.2, radius * 0.5),
        );
        draw_simple_layered_rect(painter, wrap_rect, radius * 0.1, wrap_color);

        // Visible face area (eyes only) - layered rectangle
        let eye_area_rect = Rect::from_center_size(
            Pos2::new(center.x, center.y - radius * 0.05),
            egui::vec2(radius * 1.0, radius * 0.4),
        );
        draw_layered_rect(painter, eye_area_rect, radius * 0.1, skin);
    } else {
        // When happy, show more face with wrap pulled down
        let wrap_rect = Rect::from_center_size(
            Pos2::new(center.x, center.y + radius * 0.55),
            egui::vec2(radius * 1.2, radius * 0.4),
        );
        draw_simple_layered_rect(painter, wrap_rect, radius * 0.1, wrap_color);

        // Draw more of the face - layered rectangle
        let face_rect = Rect::from_center_size(
            Pos2::new(center.x, center.y + radius * 0.1),
            egui::vec2(radius * 1.0, radius * 0.9),
        );
        draw_layered_rect(painter, face_rect, radius * 0.25, skin);
    }

    // Intense eyes - rectangular style
    let eye_y = center.y - radius * 0.05;
    let eye_offset = radius * 0.25;
    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.2, radius * 0.16),
        );
        painter.rect_filled(eye_rect, radius * 0.05, Color32::WHITE);
        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.12, radius * 0.12),
        );
        painter.rect_filled(iris_rect, radius * 0.03, Color32::from_rgb(40, 30, 20));
    }
    // Eyebrows (thick, weathered) - keep as line segments
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

    // Hair - rounded rectangle
    let hair_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - radius * 0.2),
        egui::vec2(radius * 1.6, radius * 1.4),
    );
    draw_simple_layered_rect(painter, hair_rect, radius * 0.35, hair_color);

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.15),
        egui::vec2(radius * 1.3, radius * 1.3),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Glasses - rectangular frames
    let glass_y = center.y + radius * 0.0;
    let glass_offset = radius * 0.28;
    for &x_mult in &[-1.0, 1.0] {
        let lens_x = center.x + glass_offset * x_mult;
        // Lens frame - rounded square stroke
        let lens_rect = Rect::from_center_size(
            Pos2::new(lens_x, glass_y),
            egui::vec2(radius * 0.36, radius * 0.32),
        );
        painter.rect_stroke(lens_rect, radius * 0.08, Stroke::new(2.5, glasses_color), egui::epaint::StrokeKind::Outside);
        // Eye behind lens - small rounded rectangle
        let eye_rect = Rect::from_center_size(
            Pos2::new(lens_x, glass_y),
            egui::vec2(radius * 0.12, radius * 0.14),
        );
        painter.rect_filled(eye_rect, radius * 0.03, Color32::from_rgb(60, 80, 100));
    }
    // Bridge
    painter.line_segment(
        [Pos2::new(center.x - glass_offset + radius * 0.18, glass_y), Pos2::new(center.x + glass_offset - radius * 0.18, glass_y)],
        Stroke::new(2.5, glasses_color),
    );
    // Temples
    painter.line_segment(
        [Pos2::new(center.x - glass_offset - radius * 0.18, glass_y), Pos2::new(center.x - radius * 0.7, glass_y - radius * 0.1)],
        Stroke::new(2.5, glasses_color),
    );
    painter.line_segment(
        [Pos2::new(center.x + glass_offset + radius * 0.18, glass_y), Pos2::new(center.x + radius * 0.7, glass_y - radius * 0.1)],
        Stroke::new(2.5, glasses_color),
    );

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

    // Mystical veil/hood - layered rounded rectangle
    let veil_rect = Rect::from_center_size(
        center,
        egui::vec2(radius * 1.9, radius * 1.9),
    );
    draw_layered_rect(painter, veil_rect, radius * 0.4, veil_color);

    // Face opening - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.15),
        egui::vec2(radius * 1.2, radius * 1.25),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Headpiece with gems - layered rectangle
    let headpiece_y = center.y - radius * 0.35;
    let headpiece_rect = Rect::from_center_size(
        Pos2::new(center.x, headpiece_y),
        egui::vec2(radius * 1.3, radius * 0.15),
    );
    draw_simple_layered_rect(painter, headpiece_rect, radius * 0.04, gold);

    // Center gem (third eye) - rounded square
    let center_gem_rect = Rect::from_center_size(
        Pos2::new(center.x, headpiece_y),
        egui::vec2(radius * 0.16, radius * 0.16),
    );
    painter.rect_filled(center_gem_rect, radius * 0.04, Color32::from_rgb(100, 200, 255));

    // Side gems - small rounded squares
    let left_gem_rect = Rect::from_center_size(
        Pos2::new(center.x - radius * 0.35, headpiece_y),
        egui::vec2(radius * 0.1, radius * 0.1),
    );
    painter.rect_filled(left_gem_rect, radius * 0.02, Color32::from_rgb(255, 100, 100));
    let right_gem_rect = Rect::from_center_size(
        Pos2::new(center.x + radius * 0.35, headpiece_y),
        egui::vec2(radius * 0.1, radius * 0.1),
    );
    painter.rect_filled(right_gem_rect, radius * 0.02, Color32::from_rgb(100, 255, 100));

    // Mysterious eyes - rectangular style
    let eye_y = center.y + radius * 0.05;
    let eye_offset = radius * 0.22;
    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;
        // Eye white
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.24, radius * 0.2),
        );
        painter.rect_filled(eye_rect, radius * 0.06, Color32::WHITE);
        // Purple mystical iris
        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.14, radius * 0.14),
        );
        painter.rect_filled(iris_rect, radius * 0.04, Color32::from_rgb(120, 80, 180));
        // Tiny reflection - small square
        let reflect_rect = Rect::from_center_size(
            Pos2::new(eye_x + radius * 0.03, eye_y - radius * 0.03),
            egui::vec2(radius * 0.04, radius * 0.04),
        );
        painter.rect_filled(reflect_rect, radius * 0.01, Color32::WHITE);
    }

    // Earrings - small rounded squares
    for &x_mult in &[-1.0, 1.0] {
        let earring_rect = Rect::from_center_size(
            Pos2::new(center.x + radius * 0.55 * x_mult, center.y + radius * 0.15),
            egui::vec2(radius * 0.14, radius * 0.14),
        );
        painter.rect_filled(earring_rect, radius * 0.04, gold);
    }

    // Smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.1);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.8);
    } else {
        draw_smile(painter, smile_center, radius * 0.8);
    }
}

/// Helper: Draw standard eyes (rectangular style to match camels)
fn draw_eyes(painter: &egui::Painter, center: Pos2, radius: f32, iris_color: Color32) {
    draw_rect_eyes(painter, center, radius, iris_color);
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
