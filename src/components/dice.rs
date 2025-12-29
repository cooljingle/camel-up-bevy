use bevy::prelude::*;
use rand::Rng;
use super::{CamelColor, CrazyCamelColor};

#[derive(Clone, Debug)]
pub struct RegularDie {
    pub color: CamelColor,
    pub value: Option<u8>, // 1, 2, or 3 when rolled
}

impl RegularDie {
    pub fn new(color: CamelColor) -> Self {
        Self { color, value: None }
    }

    pub fn roll(&mut self) -> u8 {
        let mut rng = rand::thread_rng();
        let value = rng.gen_range(1..=3);
        self.value = Some(value);
        value
    }

    pub fn reset(&mut self) {
        self.value = None;
    }
}

/// Represents either a regular camel die or the crazy camel die
#[derive(Clone, Debug)]
pub enum PyramidDie {
    Regular(RegularDie),
    /// The single gray crazy die - when rolled, randomly picks white or black
    /// Stores the rolled color and value after being rolled
    Crazy { rolled: Option<(CrazyCamelColor, u8)> },
}

/// Result of rolling a die from the pyramid
#[derive(Clone, Debug)]
pub enum DieRollResult {
    Regular { color: CamelColor, value: u8 },
    Crazy { color: CrazyCamelColor, value: u8 },
}

#[derive(Resource)]
pub struct Pyramid {
    pub dice: Vec<PyramidDie>,
    pub rolled_dice: Vec<PyramidDie>,
}

impl Pyramid {
    pub fn new() -> Self {
        let mut dice: Vec<PyramidDie> = CamelColor::all()
            .into_iter()
            .map(|c| PyramidDie::Regular(RegularDie::new(c)))
            .collect();

        // Add ONE crazy camel die to the pyramid (gray die shared by white/black)
        dice.push(PyramidDie::Crazy { rolled: None });

        Self {
            dice,
            rolled_dice: Vec::new(),
        }
    }

    pub fn roll_random_die(&mut self) -> Option<DieRollResult> {
        if self.dice.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.dice.len());
        let die = self.dice.remove(index);

        let result = match die {
            PyramidDie::Regular(mut regular_die) => {
                let value = regular_die.roll();
                let color = regular_die.color;
                self.rolled_dice.push(PyramidDie::Regular(regular_die));
                DieRollResult::Regular { color, value }
            }
            PyramidDie::Crazy { .. } => {
                // Roll value 1-3 and randomly pick white or black crazy camel
                let value = rng.gen_range(1..=3);
                let crazy_color = if rng.gen_bool(0.5) {
                    CrazyCamelColor::White
                } else {
                    CrazyCamelColor::Black
                };
                self.rolled_dice.push(PyramidDie::Crazy { rolled: Some((crazy_color, value)) });
                DieRollResult::Crazy { color: crazy_color, value }
            }
        };

        Some(result)
    }

    pub fn all_dice_rolled(&self) -> bool {
        // Leg ends after 5 dice are rolled (any combination of regular and crazy)
        self.rolled_dice.len() >= 5
    }

    pub fn remaining_dice_count(&self) -> usize {
        // Count all remaining dice (regular + crazy)
        self.dice.len()
    }

    pub fn remaining_regular_dice_count(&self) -> usize {
        self.dice.iter().filter(|d| matches!(d, PyramidDie::Regular(_))).count()
    }

    pub fn remaining_crazy_dice_count(&self) -> usize {
        self.dice.iter().filter(|d| matches!(d, PyramidDie::Crazy { .. })).count()
    }

    /// Check if the crazy die has been rolled this leg
    pub fn crazy_die_rolled(&self) -> bool {
        self.rolled_dice.iter().any(|d| matches!(d, PyramidDie::Crazy { .. }))
    }

    pub fn reset(&mut self) {
        for die in self.rolled_dice.drain(..) {
            match die {
                PyramidDie::Regular(mut regular_die) => {
                    regular_die.reset();
                    self.dice.push(PyramidDie::Regular(regular_die));
                }
                PyramidDie::Crazy { .. } => {
                    self.dice.push(PyramidDie::Crazy { rolled: None });
                }
            }
        }
    }
}

impl Default for Pyramid {
    fn default() -> Self {
        Self::new()
    }
}

// For crazy camels - they share a single gray die (kept for backwards compatibility)
#[derive(Resource)]
pub struct CrazyCamelDie {
    pub value: Option<u8>,
}

impl CrazyCamelDie {
    pub fn roll(&mut self) -> u8 {
        let mut rng = rand::thread_rng();
        let value = rng.gen_range(1..=3);
        self.value = Some(value);
        value
    }

    pub fn reset(&mut self) {
        self.value = None;
    }
}

impl Default for CrazyCamelDie {
    fn default() -> Self {
        Self { value: None }
    }
}
