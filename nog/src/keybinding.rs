use std::{fmt::Display, str::FromStr};

use crate::key_combination::KeyCombination;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeybindingMode {
    Global,
    Normal,
}

impl FromStr for KeybindingMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "g" => KeybindingMode::Global,
            "n" => KeybindingMode::Normal,
            _ => return Err(()),
        })
    }
}

impl Display for KeybindingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                KeybindingMode::Global => "g",
                KeybindingMode::Normal => "n",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Keybinding {
    pub key_combination: KeyCombination,
    pub mode: KeybindingMode,
}

impl Keybinding {
    pub fn get_id(&self) -> usize {
        self.key_combination.get_id()
    }
}

impl Display for Keybinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key_combination.to_string())
    }
}
