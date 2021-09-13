use std::{fmt::Display, str::FromStr};

use crate::key_combination::KeyCombination;

#[derive(Debug, Clone, Copy)]
pub enum KeybindingMode {
    Global,
    Workspace,
    Normal,
}

impl FromStr for KeybindingMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "g" => KeybindingMode::Global,
            "w" => KeybindingMode::Workspace,
            "n" => KeybindingMode::Normal,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Keybinding {
    pub key_combination: KeyCombination,
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
