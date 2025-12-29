use bevy::prelude::*;

/// Racing camel colors (Second Edition)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CamelColor {
    Blue,
    Green,
    Red,
    Yellow,
    Purple,
}

impl CamelColor {
    pub fn all() -> [CamelColor; 5] {
        [
            CamelColor::Blue,
            CamelColor::Green,
            CamelColor::Red,
            CamelColor::Yellow,
            CamelColor::Purple,
        ]
    }

    pub fn to_bevy_color(&self) -> Color {
        match self {
            CamelColor::Blue => Color::srgb(0.2, 0.4, 0.9),
            CamelColor::Green => Color::srgb(0.2, 0.8, 0.3),
            CamelColor::Red => Color::srgb(0.9, 0.2, 0.2),
            CamelColor::Yellow => Color::srgb(0.95, 0.9, 0.2),
            CamelColor::Purple => Color::srgb(0.6, 0.2, 0.8),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrazyCamelColor {
    Black,
    White,
}

impl CrazyCamelColor {
    pub fn all() -> [CrazyCamelColor; 2] {
        [CrazyCamelColor::Black, CrazyCamelColor::White]
    }

    pub fn to_bevy_color(&self) -> Color {
        match self {
            CrazyCamelColor::Black => Color::srgb(0.15, 0.15, 0.15),
            CrazyCamelColor::White => Color::srgb(0.95, 0.95, 0.95),
        }
    }
}

#[derive(Component)]
pub struct Camel {
    pub color: CamelColor,
}

#[derive(Component)]
pub struct CrazyCamel {
    pub color: CrazyCamelColor,
}

#[derive(Component)]
pub struct BoardPosition {
    pub space_index: u8,
    pub stack_position: u8, // 0 = bottom of stack
}

#[derive(Component)]
pub struct CamelSprite;
