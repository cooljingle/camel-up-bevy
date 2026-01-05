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
fn draw_layered_rect(painter: &egui::Painter, rect: Rect, rounding: f32, base_color: Color32) {
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
        let eye_rect =
            Rect::from_center_size(Pos2::new(eye_x, eye_y), egui::vec2(eye_width, eye_height));
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
    SnakeCharmer = 8,
    Sultan = 9,
    Priestess = 10,
    Archaeologist = 11,
    Vizier = 12,
    Guard = 13,
    Dancer = 14,
    Pirate = 15,
}

impl CharacterId {
    /// Get character from index (0-15), wrapping if needed
    pub fn from_index(index: usize) -> Self {
        match index % 16 {
            0 => Self::DesertExplorer,
            1 => Self::Merchant,
            2 => Self::Princess,
            3 => Self::Jockey,
            4 => Self::Pharaoh,
            5 => Self::Nomad,
            6 => Self::Scholar,
            7 => Self::FortuneTeller,
            8 => Self::SnakeCharmer,
            9 => Self::Sultan,
            10 => Self::Priestess,
            11 => Self::Archaeologist,
            12 => Self::Vizier,
            13 => Self::Guard,
            14 => Self::Dancer,
            _ => Self::Pirate,
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
            Self::SnakeCharmer => "Snake Charmer",
            Self::Sultan => "Sultan",
            Self::Priestess => "Priestess",
            Self::Archaeologist => "Archaeologist",
            Self::Vizier => "Vizier",
            Self::Guard => "Guard",
            Self::Dancer => "Dancer",
            Self::Pirate => "Pirate",
        }
    }

    /// Get a list of thematic names for this character type
    pub fn thematic_names(&self) -> &'static [&'static str] {
        match self {
            Self::DesertExplorer => &[
                "Sandy", "Indy", "Rex", "Dusty", "Marco", "Sahara", "Dunes", "Compass",
            ],
            Self::Merchant => &[
                "Hakim", "Jabari", "Omar", "Rashid", "Tariq", "Bazaar", "Silk", "Spice",
            ],
            Self::Princess => &[
                "Nefertiti",
                "Cleo",
                "Isis",
                "Amira",
                "Jasmine",
                "Lotus",
                "Pearl",
                "Sapphire",
            ],
            Self::Jockey => &[
                "Flash", "Speedy", "Blaze", "Dash", "Swift", "Bolt", "Thunder", "Rocket",
            ],
            Self::Pharaoh => &[
                "Ramses", "Tut", "Khufu", "Osiris", "Ra", "Anubis", "Horus", "Sphinx",
            ],
            Self::Nomad => &[
                "Bedouin", "Sahir", "Zephyr", "Dune", "Sirocco", "Mirage", "Wanderer", "Breeze",
            ],
            Self::Scholar => &[
                "Thoth", "Scribe", "Ptolemy", "Archie", "Sage", "Newton", "Wisdom", "Scroll",
            ],
            Self::FortuneTeller => &[
                "Oracle", "Sybil", "Cass", "Pythia", "Esme", "Zara", "Mystic", "Tarot",
            ],
            Self::SnakeCharmer => &[
                "Naja", "Cobra", "Charmer", "Sway", "Viper", "Mystic", "Serpent", "Asp",
            ],
            Self::Sultan => &[
                "Malik", "Suleiman", "Amir", "Rashid", "Crown", "Noble", "Reign", "Taj",
            ],
            Self::Priestess => &[
                "Isis", "Bastet", "Nefertari", "Luna", "Divine", "Celestia", "Hathor", "Sekhmet",
            ],
            Self::Archaeologist => &[
                "Carter", "Indiana", "Petra", "Digger", "Relic", "Shard", "Quest", "Tomb",
            ],
            Self::Vizier => &[
                "Wisdom", "Jafar", "Solomon", "Elder", "Counsel", "Sage", "Mentor", "Guide",
            ],
            Self::Guard => &[
                "Shield", "Sentinel", "Vigil", "Steel", "Bastion", "Ward", "Protector", "Watch",
            ],
            Self::Dancer => &[
                "Shimmer", "Twirl", "Grace", "Silk", "Rhythm", "Sway", "Salome", "Zara",
            ],
            Self::Pirate => &[
                "Corsair", "Captain", "Salt", "Reef", "Tide", "Cutlass", "Wave", "Buccaneer",
            ],
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
pub fn draw_avatar(
    painter: &egui::Painter,
    rect: Rect,
    character: CharacterId,
    border_color: Option<Color32>,
) {
    draw_avatar_with_expression(painter, rect, character, border_color, false)
}

/// Draw a character avatar with optional happy expression (for winners)
pub fn draw_avatar_with_expression(
    painter: &egui::Painter,
    rect: Rect,
    character: CharacterId,
    border_color: Option<Color32>,
    happy: bool,
) {
    let center = rect.center();
    let size = rect.width().min(rect.height());
    let radius = size * 0.45;

    // Draw border/background if specified (rounded rectangle to match camel style)
    if let Some(border) = border_color {
        let border_rect =
            Rect::from_center_size(center, egui::vec2(radius * 2.0 + 6.0, radius * 2.0 + 6.0));
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
        CharacterId::SnakeCharmer => draw_snake_charmer(painter, center, radius, happy),
        CharacterId::Sultan => draw_sultan(painter, center, radius, happy),
        CharacterId::Priestess => draw_priestess(painter, center, radius, happy),
        CharacterId::Archaeologist => draw_archaeologist(painter, center, radius, happy),
        CharacterId::Vizier => draw_vizier(painter, center, radius, happy),
        CharacterId::Guard => draw_guard(painter, center, radius, happy),
        CharacterId::Dancer => draw_dancer(painter, center, radius, happy),
        CharacterId::Pirate => draw_pirate(painter, center, radius, happy),
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

    // Stubble/5 o'clock shadow - darker skin tone area around jaw/chin
    let stubble_color = Color32::from_rgb(180, 145, 105);
    let stubble_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.5),
        egui::vec2(radius * 1.1, radius * 0.55),
    );
    painter.rect_filled(stubble_rect, radius * 0.2, stubble_color);

    // Eyebrows (thick, adventurous)
    let eye_y = center.y + radius * 0.1;
    let eye_offset = radius * 0.22;
    let brow_color = Color32::from_rgb(90, 60, 30);
    // Left eyebrow
    let left_brow_rect = Rect::from_center_size(
        Pos2::new(center.x - eye_offset, eye_y - radius * 0.16),
        egui::vec2(radius * 0.28, radius * 0.08),
    );
    painter.rect_filled(left_brow_rect, radius * 0.02, brow_color);
    // Right eyebrow
    let right_brow_rect = Rect::from_center_size(
        Pos2::new(center.x + eye_offset, eye_y - radius * 0.16),
        egui::vec2(radius * 0.28, radius * 0.08),
    );
    painter.rect_filled(right_brow_rect, radius * 0.02, brow_color);

    // Eyes (positioned lower)
    let eye_center = Pos2::new(center.x, center.y + radius * 0.1);
    draw_rect_eyes(painter, eye_center, radius, Color32::from_rgb(80, 60, 40));

    // Smile - big for happy, normal otherwise (positioned lower)
    let smile_center = Pos2::new(center.x, center.y + radius * 0.13);
    if happy {
        draw_big_smile(painter, smile_center, radius);
    } else {
        draw_smile(painter, smile_center, radius);
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
    let eye_y = center.y + radius * 0.08;
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
    let smile_center = Pos2::new(center.x, center.y + radius * 0.10);
    if happy {
        draw_big_smile(painter, smile_center, radius);
    } else {
        draw_smile(painter, smile_center, radius);
    }
}

/// Princess - Tiara, elegant, long eyelashes
fn draw_princess(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_LIGHT;
    let hair_color = Color32::from_rgb(60, 30, 10); // Dark brown
    let tiara_color = Color32::from_rgb(255, 215, 0); // Gold

    // Hair cap - covers top and sides of head
    let hair_cap_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - radius * 0.3),
        egui::vec2(radius * 1.9, radius * 1.3),
    );
    draw_simple_layered_rect(painter, hair_cap_rect, radius * 0.4, hair_color);

    // Flowing hair strands on left side (wider to frame larger face)
    for (x_offset, width_mult) in [(-0.85, 0.45), (-0.55, 0.4)] {
        let strand_rect = Rect::from_min_size(
            Pos2::new(center.x + radius * x_offset, center.y - radius * 0.2),
            egui::vec2(radius * width_mult, radius * 1.3),
        );
        draw_simple_layered_rect(painter, strand_rect, radius * 0.12, hair_color);
    }

    // Flowing hair strands on right side
    for (x_offset, width_mult) in [(0.4, 0.45), (0.15, 0.4)] {
        let strand_rect = Rect::from_min_size(
            Pos2::new(center.x + radius * x_offset, center.y - radius * 0.2),
            egui::vec2(radius * width_mult, radius * 1.3),
        );
        draw_simple_layered_rect(painter, strand_rect, radius * 0.12, hair_color);
    }

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
    painter.add(egui::Shape::convex_polygon(
        peak_points.to_vec(),
        tiara_color,
        Stroke::NONE,
    ));
    // Side peaks
    for offset in [-0.35, 0.35] {
        let peak = [
            Pos2::new(center.x + radius * offset, tiara_y - radius * 0.1),
            Pos2::new(center.x + radius * (offset - 0.1), tiara_y + radius * 0.1),
            Pos2::new(center.x + radius * (offset + 0.1), tiara_y + radius * 0.1),
        ];
        painter.add(egui::Shape::convex_polygon(
            peak.to_vec(),
            tiara_color,
            Stroke::NONE,
        ));
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
            [
                Pos2::new(lash_x_left, eye_y - radius * 0.12),
                Pos2::new(lash_x_left, eye_y - radius * 0.2),
            ],
            Stroke::new(1.5, Color32::BLACK),
        );
        painter.line_segment(
            [
                Pos2::new(lash_x_right, eye_y - radius * 0.12),
                Pos2::new(lash_x_right, eye_y - radius * 0.2),
            ],
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
        painter.rect_stroke(
            lens_rect,
            radius * 0.08,
            Stroke::new(2.0, goggle_color),
            egui::epaint::StrokeKind::Outside,
        );
    }

    // Eyes
    draw_eyes(
        painter,
        Pos2::new(center.x, center.y + radius * 0.1),
        radius * 0.8,
        Color32::from_rgb(60, 120, 60),
    );

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
    painter.add(egui::Shape::convex_polygon(
        flap_points_left.to_vec(),
        headdress_color,
        Stroke::NONE,
    ));
    painter.add(egui::Shape::convex_polygon(
        flap_points_right.to_vec(),
        headdress_color,
        Stroke::NONE,
    ));

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
        [
            Pos2::new(center.x - eye_offset - radius * 0.1, eye_y),
            Pos2::new(center.x - eye_offset - radius * 0.25, eye_y + radius * 0.1),
        ],
        Stroke::new(2.0, Color32::BLACK),
    );
    painter.line_segment(
        [
            Pos2::new(center.x + eye_offset + radius * 0.1, eye_y),
            Pos2::new(center.x + eye_offset + radius * 0.25, eye_y + radius * 0.1),
        ],
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
    painter.add(egui::Shape::convex_polygon(
        left_drape.to_vec(),
        scarf_color,
        Stroke::NONE,
    ));
    painter.add(egui::Shape::convex_polygon(
        right_drape.to_vec(),
        scarf_color,
        Stroke::NONE,
    ));

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
        [
            Pos2::new(center.x - eye_offset - radius * 0.12, eye_y - radius * 0.15),
            Pos2::new(center.x - eye_offset + radius * 0.12, eye_y - radius * 0.18),
        ],
        Stroke::new(3.0, Color32::from_rgb(30, 20, 10)),
    );
    painter.line_segment(
        [
            Pos2::new(center.x + eye_offset - radius * 0.12, eye_y - radius * 0.18),
            Pos2::new(center.x + eye_offset + radius * 0.12, eye_y - radius * 0.15),
        ],
        Stroke::new(3.0, Color32::from_rgb(30, 20, 10)),
    );

    // Smile when happy (visible since wrap is pulled down)
    if happy {
        draw_big_smile(
            painter,
            Pos2::new(center.x, center.y + radius * 0.1),
            radius * 0.7,
        );
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
        painter.rect_stroke(
            lens_rect,
            radius * 0.08,
            Stroke::new(2.5, glasses_color),
            egui::epaint::StrokeKind::Outside,
        );
        // Eye behind lens - small rounded rectangle
        let eye_rect = Rect::from_center_size(
            Pos2::new(lens_x, glass_y),
            egui::vec2(radius * 0.12, radius * 0.14),
        );
        painter.rect_filled(eye_rect, radius * 0.03, Color32::from_rgb(60, 80, 100));
    }
    // Bridge
    painter.line_segment(
        [
            Pos2::new(center.x - glass_offset + radius * 0.18, glass_y),
            Pos2::new(center.x + glass_offset - radius * 0.18, glass_y),
        ],
        Stroke::new(2.5, glasses_color),
    );
    // Temples
    painter.line_segment(
        [
            Pos2::new(center.x - glass_offset - radius * 0.18, glass_y),
            Pos2::new(center.x - radius * 0.7, glass_y - radius * 0.1),
        ],
        Stroke::new(2.5, glasses_color),
    );
    painter.line_segment(
        [
            Pos2::new(center.x + glass_offset + radius * 0.18, glass_y),
            Pos2::new(center.x + radius * 0.7, glass_y - radius * 0.1),
        ],
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

    // Veil cap - covers top and sides of head
    let veil_cap_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - radius * 0.3),
        egui::vec2(radius * 1.9, radius * 1.3),
    );
    draw_layered_rect(painter, veil_cap_rect, radius * 0.4, veil_color);

    // Flowing veil strands on left side (wider/more flowing than princess)
    for (x_offset, width_mult) in [(-0.75, 0.4), (-0.45, 0.35)] {
        let strand_rect = Rect::from_min_size(
            Pos2::new(center.x + radius * x_offset, center.y - radius * 0.2),
            egui::vec2(radius * width_mult, radius * 1.15),
        );
        draw_simple_layered_rect(painter, strand_rect, radius * 0.12, veil_color);
    }

    // Flowing veil strands on right side
    for (x_offset, width_mult) in [(0.35, 0.4), (0.1, 0.35)] {
        let strand_rect = Rect::from_min_size(
            Pos2::new(center.x + radius * x_offset, center.y - radius * 0.2),
            egui::vec2(radius * width_mult, radius * 1.15),
        );
        draw_simple_layered_rect(painter, strand_rect, radius * 0.12, veil_color);
    }

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
    painter.rect_filled(
        center_gem_rect,
        radius * 0.04,
        Color32::from_rgb(100, 200, 255),
    );

    // Side gems - small rounded squares
    let left_gem_rect = Rect::from_center_size(
        Pos2::new(center.x - radius * 0.35, headpiece_y),
        egui::vec2(radius * 0.1, radius * 0.1),
    );
    painter.rect_filled(
        left_gem_rect,
        radius * 0.02,
        Color32::from_rgb(255, 100, 100),
    );
    let right_gem_rect = Rect::from_center_size(
        Pos2::new(center.x + radius * 0.35, headpiece_y),
        egui::vec2(radius * 0.1, radius * 0.1),
    );
    painter.rect_filled(
        right_gem_rect,
        radius * 0.02,
        Color32::from_rgb(100, 255, 100),
    );

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

/// Snake Charmer - Jeweled forehead chain, heavy kohl, nose ring
fn draw_snake_charmer(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_MEDIUM;
    let hair_color = Color32::from_rgb(20, 10, 5); // Very dark hair
    let gold = Color32::from_rgb(255, 200, 50);
    let jewel_color = Color32::from_rgb(150, 50, 200); // Purple jewel

    // Hair covering top and sides
    let hair_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - radius * 0.2),
        egui::vec2(radius * 1.8, radius * 1.4),
    );
    draw_simple_layered_rect(painter, hair_rect, radius * 0.4, hair_color);

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.15),
        egui::vec2(radius * 1.3, radius * 1.4),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Jeweled forehead chain (tikka) - chain across forehead
    let chain_y = center.y - radius * 0.25;
    let chain_rect = Rect::from_center_size(
        Pos2::new(center.x, chain_y),
        egui::vec2(radius * 1.0, radius * 0.08),
    );
    draw_simple_layered_rect(painter, chain_rect, radius * 0.02, gold);

    // Center forehead jewel
    let jewel_rect = Rect::from_center_size(
        Pos2::new(center.x, chain_y),
        egui::vec2(radius * 0.18, radius * 0.18),
    );
    painter.rect_filled(jewel_rect, radius * 0.05, jewel_color);

    // Heavy kohl-lined eyes
    let eye_y = center.y + radius * 0.05;
    let eye_offset = radius * 0.22;
    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;

        // Kohl outline (larger rectangle)
        let kohl_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.3, radius * 0.24),
        );
        painter.rect_filled(kohl_rect, radius * 0.08, Color32::BLACK);

        // Eye white
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.24, radius * 0.18),
        );
        painter.rect_filled(eye_rect, radius * 0.06, Color32::WHITE);

        // Dark iris
        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.14, radius * 0.14),
        );
        painter.rect_filled(iris_rect, radius * 0.04, Color32::from_rgb(40, 20, 10));
    }

    // Nose ring - small circle on left side
    let nose_ring_center = Pos2::new(center.x - radius * 0.15, center.y + radius * 0.22);
    painter.circle_filled(nose_ring_center, radius * 0.08, gold);
    painter.circle_stroke(
        nose_ring_center,
        radius * 0.08,
        Stroke::new(1.5, darken(gold, 0.3)),
    );

    // Smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.15);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.8);
    } else {
        draw_smile(painter, smile_center, radius * 0.8);
    }
}

/// Sultan - Tall ornate turban with jewel, curled mustache
fn draw_sultan(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_TAN;
    let turban_color = Color32::from_rgb(150, 30, 30); // Deep red
    let turban_wrap = Color32::from_rgb(200, 180, 100); // Gold wrap
    let gold = Color32::from_rgb(255, 200, 50);

    // Tall turban base
    let turban_y = center.y - radius * 0.55;
    let turban_rect = Rect::from_center_size(
        Pos2::new(center.x, turban_y),
        egui::vec2(radius * 1.4, radius * 1.0),
    );
    draw_layered_rect(painter, turban_rect, radius * 0.25, turban_color);

    // Turban wrap bands (horizontal stripes)
    for i in 0..3 {
        let band_y = turban_y - radius * 0.3 + (i as f32 * radius * 0.25);
        let band_rect = Rect::from_center_size(
            Pos2::new(center.x, band_y),
            egui::vec2(radius * 1.35, radius * 0.12),
        );
        painter.rect_filled(band_rect, radius * 0.03, turban_wrap);
    }

    // Large center jewel on turban
    let jewel_rect = Rect::from_center_size(
        Pos2::new(center.x, turban_y + radius * 0.35),
        egui::vec2(radius * 0.28, radius * 0.28),
    );
    painter.rect_filled(jewel_rect, radius * 0.08, Color32::from_rgb(50, 200, 100)); // Emerald

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.1),
        egui::vec2(radius * 1.3, radius * 1.3),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Curled mustache - two curved rectangles
    let mustache_y = center.y + radius * 0.25;
    let mustache_color = Color32::from_rgb(30, 20, 10);

    // Left curl
    for i in 0..3 {
        let curl_x = center.x - radius * 0.15 - (i as f32 * radius * 0.12);
        let curl_y = mustache_y - (i as f32 * radius * 0.08);
        let curl_rect = Rect::from_center_size(
            Pos2::new(curl_x, curl_y),
            egui::vec2(radius * 0.15, radius * 0.12),
        );
        painter.rect_filled(curl_rect, radius * 0.04, mustache_color);
    }

    // Right curl
    for i in 0..3 {
        let curl_x = center.x + radius * 0.15 + (i as f32 * radius * 0.12);
        let curl_y = mustache_y - (i as f32 * radius * 0.08);
        let curl_rect = Rect::from_center_size(
            Pos2::new(curl_x, curl_y),
            egui::vec2(radius * 0.15, radius * 0.12),
        );
        painter.rect_filled(curl_rect, radius * 0.04, mustache_color);
    }

    // Regal eyes
    let eye_y = center.y + radius * 0.05;
    draw_rect_eyes(painter, Pos2::new(center.x, eye_y), radius, Color32::from_rgb(60, 40, 20));

    // Smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.12);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.7);
    } else {
        draw_smile(painter, smile_center, radius * 0.7);
    }
}

/// Priestess - Cobra/vulture headdress, kohl-lined eyes, ankh earrings
fn draw_priestess(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_TAN;
    let headdress_color = Color32::from_rgb(200, 180, 50); // Gold
    let headdress_blue = Color32::from_rgb(30, 60, 120); // Royal blue stripes
    let gold = Color32::from_rgb(255, 200, 50);

    // Headdress base (like nemes but rounder)
    let head_top = center.y - radius * 0.5;
    let headdress_rect = Rect::from_center_size(
        Pos2::new(center.x, head_top),
        egui::vec2(radius * 1.6, radius * 1.0),
    );
    draw_layered_rect(painter, headdress_rect, radius * 0.25, headdress_color);

    // Blue stripes on headdress (vertical)
    for i in -2..=2 {
        let stripe_x = center.x + (i as f32 * radius * 0.3);
        let stripe_rect = Rect::from_center_size(
            Pos2::new(stripe_x, head_top),
            egui::vec2(radius * 0.12, radius * 0.9),
        );
        painter.rect_filled(stripe_rect, radius * 0.03, headdress_blue);
    }

    // Cobra uraeus (raised cobra on forehead)
    let cobra_points = [
        Pos2::new(center.x, head_top - radius * 0.3), // Top of hood
        Pos2::new(center.x - radius * 0.15, head_top + radius * 0.1), // Left hood
        Pos2::new(center.x - radius * 0.08, head_top + radius * 0.3), // Left body
        Pos2::new(center.x + radius * 0.08, head_top + radius * 0.3), // Right body
        Pos2::new(center.x + radius * 0.15, head_top + radius * 0.1), // Right hood
    ];
    painter.add(egui::Shape::convex_polygon(
        cobra_points.to_vec(),
        gold,
        Stroke::new(1.0, darken(gold, 0.4)),
    ));

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.15),
        egui::vec2(radius * 1.2, radius * 1.3),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Dramatic kohl-lined eyes
    let eye_y = center.y + radius * 0.05;
    let eye_offset = radius * 0.22;
    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;

        // Eye white
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.26, radius * 0.2),
        );
        painter.rect_filled(eye_rect, radius * 0.06, Color32::WHITE);

        // Iris
        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.14, radius * 0.16),
        );
        painter.rect_filled(iris_rect, radius * 0.04, Color32::from_rgb(40, 80, 40)); // Green
    }

    // Heavy eyeliner wings
    painter.line_segment(
        [
            Pos2::new(center.x - eye_offset - radius * 0.13, eye_y),
            Pos2::new(center.x - eye_offset - radius * 0.3, eye_y + radius * 0.12),
        ],
        Stroke::new(3.0, Color32::BLACK),
    );
    painter.line_segment(
        [
            Pos2::new(center.x + eye_offset + radius * 0.13, eye_y),
            Pos2::new(center.x + eye_offset + radius * 0.3, eye_y + radius * 0.12),
        ],
        Stroke::new(3.0, Color32::BLACK),
    );

    // Ankh earrings - simplified ankh shape
    for &x_mult in &[-1.0, 1.0] {
        let earring_x = center.x + radius * 0.6 * x_mult;
        let earring_y = center.y + radius * 0.2;

        // Ankh loop (circle)
        painter.circle_stroke(
            Pos2::new(earring_x, earring_y - radius * 0.08),
            radius * 0.08,
            Stroke::new(2.0, gold),
        );
        // Ankh cross (vertical and horizontal lines)
        painter.line_segment(
            [
                Pos2::new(earring_x, earring_y),
                Pos2::new(earring_x, earring_y + radius * 0.2),
            ],
            Stroke::new(2.5, gold),
        );
        painter.line_segment(
            [
                Pos2::new(earring_x - radius * 0.08, earring_y + radius * 0.05),
                Pos2::new(earring_x + radius * 0.08, earring_y + radius * 0.05),
            ],
            Stroke::new(2.5, gold),
        );
    }

    // Serene smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.15);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.7);
    } else {
        draw_smile(painter, smile_center, radius * 0.7);
    }
}

/// Archaeologist - Pith helmet, aviator goggles on forehead, stubble
fn draw_archaeologist(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_LIGHT;
    let helmet_color = Color32::from_rgb(220, 210, 180); // Off-white pith helmet
    let goggle_color = Color32::from_rgb(100, 70, 40); // Leather brown
    let lens_color = Color32::from_rgb(180, 200, 220); // Light blue lens

    // Pith helmet (colonial style)
    let helmet_y = center.y - radius * 0.35;

    // Helmet crown (tall dome)
    let crown_rect = Rect::from_center_size(
        Pos2::new(center.x, helmet_y - radius * 0.25),
        egui::vec2(radius * 1.2, radius * 0.7),
    );
    draw_layered_rect(painter, crown_rect, radius * 0.25, helmet_color);

    // Helmet brim
    let brim_rect = Rect::from_center_size(
        Pos2::new(center.x, helmet_y + radius * 0.05),
        egui::vec2(radius * 1.7, radius * 0.25),
    );
    draw_simple_layered_rect(painter, brim_rect, radius * 0.08, helmet_color);

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.15),
        egui::vec2(radius * 1.3, radius * 1.4),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Aviator goggles pushed up on forehead
    let goggle_y = center.y - radius * 0.22;

    // Goggle strap
    let strap_rect = Rect::from_center_size(
        Pos2::new(center.x, goggle_y),
        egui::vec2(radius * 1.4, radius * 0.1),
    );
    painter.rect_filled(strap_rect, radius * 0.02, goggle_color);

    // Goggle lenses (pushed up)
    for &x_mult in &[-1.0, 1.0] {
        let lens_x = center.x + radius * 0.28 * x_mult;
        let lens_rect = Rect::from_center_size(
            Pos2::new(lens_x, goggle_y),
            egui::vec2(radius * 0.3, radius * 0.26),
        );
        painter.rect_filled(lens_rect, radius * 0.08, lens_color);
        painter.rect_stroke(
            lens_rect,
            radius * 0.08,
            Stroke::new(2.0, goggle_color),
            egui::epaint::StrokeKind::Outside,
        );
    }

    // Stubble (5 o'clock shadow)
    let stubble_color = Color32::from_rgb(200, 180, 160);
    let stubble_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.45),
        egui::vec2(radius * 1.0, radius * 0.6),
    );
    painter.rect_filled(stubble_rect, radius * 0.2, stubble_color);

    // Determined eyes
    let eye_y = center.y + radius * 0.08;
    draw_rect_eyes(painter, Pos2::new(center.x, eye_y), radius, Color32::from_rgb(100, 80, 60));

    // Excited/determined smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.12);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.8);
    } else {
        draw_smile(painter, smile_center, radius * 0.8);
    }
}

/// Vizier - Long braided beard, tall conical hat, wise expression
fn draw_vizier(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_LIGHT;
    let hat_color = Color32::from_rgb(60, 40, 100); // Deep purple
    let beard_color = Color32::from_rgb(200, 200, 200); // White/grey beard
    let gold = Color32::from_rgb(255, 200, 50);

    // Tall conical hat
    let hat_base_y = center.y - radius * 0.35;

    // Hat as trapezoid (narrower at top)
    let hat_points = [
        Pos2::new(center.x, hat_base_y - radius * 0.8), // Top point
        Pos2::new(center.x - radius * 0.35, hat_base_y), // Bottom left
        Pos2::new(center.x + radius * 0.35, hat_base_y), // Bottom right
    ];
    painter.add(egui::Shape::convex_polygon(
        hat_points.to_vec(),
        hat_color,
        Stroke::NONE,
    ));

    // Add layering to hat
    let hat_rect = Rect::from_center_size(
        Pos2::new(center.x, hat_base_y - radius * 0.4),
        egui::vec2(radius * 0.6, radius * 0.7),
    );
    draw_simple_layered_rect(painter, hat_rect, radius * 0.15, hat_color);

    // Hat band (gold)
    let band_rect = Rect::from_center_size(
        Pos2::new(center.x, hat_base_y),
        egui::vec2(radius * 1.3, radius * 0.12),
    );
    painter.rect_filled(band_rect, radius * 0.03, gold);

    // Long braided beard
    let beard_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.55),
        egui::vec2(radius * 0.7, radius * 1.1),
    );
    draw_layered_rect(painter, beard_rect, radius * 0.15, beard_color);

    // Beard braids (horizontal lines to show braiding)
    for i in 0..5 {
        let braid_y = center.y + radius * 0.3 + (i as f32 * radius * 0.18);
        painter.line_segment(
            [
                Pos2::new(center.x - radius * 0.25, braid_y),
                Pos2::new(center.x + radius * 0.25, braid_y),
            ],
            Stroke::new(1.5, darken(beard_color, 0.2)),
        );
    }

    // Beard beads (small gold circles)
    for i in 1..=3 {
        let bead_y = center.y + radius * 0.3 + (i as f32 * radius * 0.25);
        painter.circle_filled(Pos2::new(center.x, bead_y), radius * 0.06, gold);
    }

    // Face - layered rounded rectangle (wrinkled/aged)
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.05),
        egui::vec2(radius * 1.2, radius * 1.1),
    );
    draw_layered_rect(painter, face_rect, radius * 0.3, skin);

    // Wrinkle lines (age)
    for &offset in &[-0.15, 0.15] {
        painter.line_segment(
            [
                Pos2::new(center.x + radius * offset, center.y - radius * 0.1),
                Pos2::new(center.x + radius * offset, center.y + radius * 0.15),
            ],
            Stroke::new(1.0, darken(skin, 0.15)),
        );
    }

    // Wise eyes
    let eye_y = center.y + radius * 0.0;
    draw_rect_eyes(painter, Pos2::new(center.x, eye_y), radius * 0.9, Color32::from_rgb(80, 80, 120));

    // Knowing smile
    let smile_center = Pos2::new(center.x, center.y + radius * 0.08);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.6);
    } else {
        draw_smile(painter, smile_center, radius * 0.6);
    }
}

/// Guard - Metal helmet with nose guard, battle scar, fierce eyes
fn draw_guard(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_TAN;
    let helmet_color = Color32::from_rgb(140, 140, 150); // Steel grey
    let helmet_trim = Color32::from_rgb(180, 160, 80); // Brass trim

    // Helmet dome
    let helmet_y = center.y - radius * 0.3;
    let helmet_rect = Rect::from_center_size(
        Pos2::new(center.x, helmet_y),
        egui::vec2(radius * 1.6, radius * 1.2),
    );
    draw_layered_rect(painter, helmet_rect, radius * 0.35, helmet_color);

    // Helmet trim/band
    let trim_rect = Rect::from_center_size(
        Pos2::new(center.x, helmet_y + radius * 0.4),
        egui::vec2(radius * 1.55, radius * 0.15),
    );
    painter.rect_filled(trim_rect, radius * 0.03, helmet_trim);

    // Face opening
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.2),
        egui::vec2(radius * 1.1, radius * 1.2),
    );
    draw_layered_rect(painter, face_rect, radius * 0.3, skin);

    // Nose guard (vertical strip down center)
    let nose_guard_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.1),
        egui::vec2(radius * 0.18, radius * 0.7),
    );
    draw_simple_layered_rect(painter, nose_guard_rect, radius * 0.04, helmet_color);

    // Battle scar across cheek
    let scar_y = center.y + radius * 0.25;
    painter.line_segment(
        [
            Pos2::new(center.x - radius * 0.45, scar_y - radius * 0.1),
            Pos2::new(center.x + radius * 0.15, scar_y + radius * 0.15),
        ],
        Stroke::new(2.5, Color32::from_rgb(160, 110, 90)),
    );

    // Fierce eyes
    let eye_y = center.y + radius * 0.05;
    let eye_offset = radius * 0.25;
    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;

        // Skip left eye if covered by nose guard
        if x_mult < 0.0 && (eye_x - center.x).abs() < radius * 0.15 {
            continue;
        }

        // Narrowed, intense eyes
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.22, radius * 0.14),
        );
        painter.rect_filled(eye_rect, radius * 0.04, Color32::WHITE);

        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.13, radius * 0.12),
        );
        painter.rect_filled(iris_rect, radius * 0.03, Color32::from_rgb(60, 40, 20));
    }

    // Thick eyebrows (fierce)
    for &x_mult in &[-1.0, 1.0] {
        let brow_x = center.x + eye_offset * x_mult;
        let brow_rect = Rect::from_center_size(
            Pos2::new(brow_x, eye_y - radius * 0.15),
            egui::vec2(radius * 0.28, radius * 0.1),
        );
        painter.rect_filled(brow_rect, radius * 0.02, Color32::from_rgb(40, 30, 20));
    }

    // Determined expression
    let smile_center = Pos2::new(center.x, center.y + radius * 0.15);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.7);
    } else {
        draw_smile(painter, smile_center, radius * 0.6);
    }
}

/// Dancer - Coin headpiece, face veil, gold nose stud, decorative eyes
fn draw_dancer(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_MEDIUM;
    let veil_color = Color32::from_rgb(120, 50, 100); // Deep magenta
    let hair_color = Color32::from_rgb(30, 15, 10);
    let gold = Color32::from_rgb(255, 200, 50);

    // Hair visible at top
    let hair_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - radius * 0.3),
        egui::vec2(radius * 1.5, radius * 1.0),
    );
    draw_simple_layered_rect(painter, hair_rect, radius * 0.35, hair_color);

    // Coin headpiece across forehead
    let headpiece_y = center.y - radius * 0.35;

    // Headpiece band
    let band_rect = Rect::from_center_size(
        Pos2::new(center.x, headpiece_y),
        egui::vec2(radius * 1.3, radius * 0.12),
    );
    draw_simple_layered_rect(painter, band_rect, radius * 0.03, gold);

    // Hanging coins
    for i in -2..=2 {
        let coin_x = center.x + (i as f32 * radius * 0.28);
        let coin_y = headpiece_y + radius * 0.18;

        // Coin (small circle)
        painter.circle_filled(Pos2::new(coin_x, coin_y), radius * 0.08, gold);
        painter.circle_stroke(
            Pos2::new(coin_x, coin_y),
            radius * 0.08,
            Stroke::new(1.0, darken(gold, 0.3)),
        );

        // Chain/connection
        painter.line_segment(
            [
                Pos2::new(coin_x, headpiece_y + radius * 0.06),
                Pos2::new(coin_x, coin_y - radius * 0.08),
            ],
            Stroke::new(1.5, gold),
        );
    }

    // Upper face visible (eyes area)
    let eye_area_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - radius * 0.05),
        egui::vec2(radius * 1.2, radius * 0.5),
    );
    draw_layered_rect(painter, eye_area_rect, radius * 0.15, skin);

    // Face veil (covers lower face)
    let veil_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.35),
        egui::vec2(radius * 1.3, radius * 0.9),
    );
    draw_simple_layered_rect(painter, veil_rect, radius * 0.2, veil_color);

    // Decorative eyes with makeup
    let eye_y = center.y - radius * 0.05;
    let eye_offset = radius * 0.24;
    for &x_mult in &[-1.0, 1.0] {
        let eye_x = center.x + eye_offset * x_mult;

        // Eye makeup (colorful shadow)
        let shadow_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.32, radius * 0.26),
        );
        painter.rect_filled(shadow_rect, radius * 0.08, Color32::from_rgb(150, 80, 120));

        // Eye white
        let eye_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.26, radius * 0.2),
        );
        painter.rect_filled(eye_rect, radius * 0.07, Color32::WHITE);

        // Iris (dark)
        let iris_rect = Rect::from_center_size(
            Pos2::new(eye_x, eye_y),
            egui::vec2(radius * 0.15, radius * 0.16),
        );
        painter.rect_filled(iris_rect, radius * 0.04, Color32::from_rgb(40, 20, 10));

        // Sparkle in eye
        let sparkle_rect = Rect::from_center_size(
            Pos2::new(eye_x + radius * 0.04, eye_y - radius * 0.04),
            egui::vec2(radius * 0.05, radius * 0.05),
        );
        painter.rect_filled(sparkle_rect, radius * 0.01, Color32::WHITE);
    }

    // Gold nose stud (visible above veil)
    let nose_y = center.y + radius * 0.15;
    painter.circle_filled(Pos2::new(center.x - radius * 0.12, nose_y), radius * 0.06, gold);

    // If happy, show eyes crinkling (smile with eyes)
    if happy {
        for &x_mult in &[-1.0, 1.0] {
            let eye_x = center.x + eye_offset * x_mult;
            // Crinkle lines
            painter.line_segment(
                [
                    Pos2::new(eye_x + radius * 0.15 * x_mult, eye_y - radius * 0.08),
                    Pos2::new(eye_x + radius * 0.22 * x_mult, eye_y - radius * 0.12),
                ],
                Stroke::new(1.5, darken(skin, 0.2)),
            );
            painter.line_segment(
                [
                    Pos2::new(eye_x + radius * 0.15 * x_mult, eye_y + radius * 0.08),
                    Pos2::new(eye_x + radius * 0.22 * x_mult, eye_y + radius * 0.12),
                ],
                Stroke::new(1.5, darken(skin, 0.2)),
            );
        }
    }
}

/// Pirate - Eye patch, bandana, gold earring, scruffy beard, roguish grin
fn draw_pirate(painter: &egui::Painter, center: Pos2, radius: f32, happy: bool) {
    let skin = SKIN_TAN;
    let bandana_color = Color32::from_rgb(150, 30, 30); // Red bandana
    let beard_color = Color32::from_rgb(60, 40, 20);
    let gold = Color32::from_rgb(255, 200, 50);

    // Bandana
    let bandana_y = center.y - radius * 0.35;
    let bandana_rect = Rect::from_center_size(
        Pos2::new(center.x, bandana_y),
        egui::vec2(radius * 1.6, radius * 0.8),
    );
    draw_layered_rect(painter, bandana_rect, radius * 0.2, bandana_color);

    // Bandana knot on side
    let knot_rect = Rect::from_center_size(
        Pos2::new(center.x + radius * 0.7, bandana_y + radius * 0.2),
        egui::vec2(radius * 0.3, radius * 0.25),
    );
    draw_simple_layered_rect(painter, knot_rect, radius * 0.08, bandana_color);

    // Scruffy beard
    let beard_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.5),
        egui::vec2(radius * 1.2, radius * 0.8),
    );
    draw_simple_layered_rect(painter, beard_rect, radius * 0.25, beard_color);

    // Beard stubble texture (small rectangles)
    for i in 0..8 {
        let stubble_x = center.x - radius * 0.4 + (i as f32 * radius * 0.12);
        let stubble_y = center.y + radius * 0.4 + ((i % 3) as f32 * radius * 0.1);
        let stubble_rect = Rect::from_center_size(
            Pos2::new(stubble_x, stubble_y),
            egui::vec2(radius * 0.06, radius * 0.08),
        );
        painter.rect_filled(stubble_rect, radius * 0.01, darken(beard_color, 0.2));
    }

    // Face - layered rounded rectangle
    let face_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + radius * 0.1),
        egui::vec2(radius * 1.3, radius * 1.2),
    );
    draw_layered_rect(painter, face_rect, radius * 0.35, skin);

    // Eye patch over left eye
    let patch_center = Pos2::new(center.x - radius * 0.25, center.y + radius * 0.05);
    let patch_rect = Rect::from_center_size(
        patch_center,
        egui::vec2(radius * 0.35, radius * 0.3),
    );
    painter.rect_filled(patch_rect, radius * 0.1, Color32::BLACK);

    // Eye patch strap
    painter.line_segment(
        [
            Pos2::new(center.x - radius * 0.45, center.y + radius * 0.05),
            Pos2::new(center.x - radius * 0.7, center.y - radius * 0.1),
        ],
        Stroke::new(2.5, Color32::from_rgb(40, 40, 40)),
    );
    painter.line_segment(
        [
            Pos2::new(center.x - radius * 0.05, center.y + radius * 0.05),
            Pos2::new(center.x + radius * 0.7, center.y - radius * 0.1),
        ],
        Stroke::new(2.5, Color32::from_rgb(40, 40, 40)),
    );

    // Good eye (right eye)
    let eye_x = center.x + radius * 0.25;
    let eye_y = center.y + radius * 0.05;
    let eye_rect = Rect::from_center_size(
        Pos2::new(eye_x, eye_y),
        egui::vec2(radius * 0.24, radius * 0.2),
    );
    painter.rect_filled(eye_rect, radius * 0.06, Color32::WHITE);

    let iris_rect = Rect::from_center_size(
        Pos2::new(eye_x, eye_y),
        egui::vec2(radius * 0.14, radius * 0.15),
    );
    painter.rect_filled(iris_rect, radius * 0.04, Color32::from_rgb(80, 60, 40));

    // Gold hoop earring (right ear)
    let earring_center = Pos2::new(center.x + radius * 0.6, center.y + radius * 0.2);
    painter.circle_stroke(
        earring_center,
        radius * 0.12,
        Stroke::new(3.0, gold),
    );

    // Roguish grin (gap-toothed smile)
    let smile_center = Pos2::new(center.x, center.y + radius * 0.12);
    if happy {
        draw_big_smile(painter, smile_center, radius * 0.8);

        // Gap in teeth
        let gap_rect = Rect::from_center_size(
            Pos2::new(center.x + radius * 0.1, smile_center.y + radius * 0.08),
            egui::vec2(radius * 0.08, radius * 0.12),
        );
        painter.rect_filled(gap_rect, radius * 0.02, Color32::from_rgb(100, 60, 60));
    } else {
        draw_smile(painter, smile_center, radius * 0.8);
    }
}

/// Draw a small crown on top of an avatar (for winner)
pub fn draw_avatar_crown(painter: &egui::Painter, rect: Rect) {
    let center = rect.center();
    let avatar_size = rect.width().min(rect.height());

    // Crown sizing relative to avatar
    const CROWN_SCALE: f32 = 0.08; // Crown is 4% of avatar size per unit
    const HEAD_OVERLAP: f32 = 0.08; // Crown overlaps 8% into avatar from top

    // Crown proportions (in scale units)
    const BASE_WIDTH: f32 = 8.0;
    const BASE_HEIGHT: f32 = 2.0;
    const POINT_HEIGHT: f32 = 4.0;
    const POINT_WIDTH: f32 = 2.5;
    const POINT_SPACING: f32 = 2.5;
    const NUM_POINTS: i32 = 3;

    let scale = avatar_size * CROWN_SCALE;
    let crown_center = Pos2::new(center.x, rect.min.y + avatar_size * HEAD_OVERLAP);

    // Crown colors
    let gold = Color32::from_rgb(255, 215, 0);
    let outline = Color32::from_rgb(60, 40, 0); // Dark outline for visibility
    let jewel_red = Color32::from_rgb(220, 50, 50);
    let jewel_blue = Color32::from_rgb(50, 100, 220);

    // Crown base (rectangle)
    let base_offset_y = 1.5 * scale;
    let base_rect = Rect::from_center_size(
        crown_center + egui::vec2(0.0, base_offset_y),
        egui::vec2(BASE_WIDTH * scale, BASE_HEIGHT * scale),
    );
    painter.rect_filled(base_rect, scale, gold);
    painter.rect_stroke(base_rect, scale, egui::Stroke::new(scale * 0.4, outline), egui::epaint::StrokeKind::Outside);

    // Crown points (triangles)
    let point_base_y = crown_center.y - scale;

    for i in 0..NUM_POINTS {
        let x_offset = (i as f32 - 1.0) * POINT_SPACING * scale;
        let point_center = Pos2::new(crown_center.x + x_offset, point_base_y);

        let points = vec![
            Pos2::new(point_center.x, point_center.y - POINT_HEIGHT * scale), // Top
            Pos2::new(point_center.x - POINT_WIDTH * scale / 2.0, point_center.y), // Bottom left
            Pos2::new(point_center.x + POINT_WIDTH * scale / 2.0, point_center.y), // Bottom right
        ];
        painter.add(egui::Shape::convex_polygon(
            points,
            gold,
            egui::Stroke::new(0.8 * scale, outline),
        ));
    }

    // Gems on crown points
    let gem_colors = [jewel_red, jewel_blue, jewel_red];
    let gem_offset_y = 2.0 * scale;
    for i in 0..NUM_POINTS {
        let x_offset = (i as f32 - 1.0) * POINT_SPACING * scale;
        let gem_pos = Pos2::new(crown_center.x + x_offset, point_base_y - gem_offset_y);
        painter.circle_filled(gem_pos, scale, gem_colors[i as usize]);
        painter.circle_stroke(gem_pos, scale, egui::Stroke::new(scale * 0.3, outline));
    }
}
