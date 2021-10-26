use std::{fmt::Display, str::FromStr};

use itertools::Itertools;

use crate::{key::Key, modifiers::Modifiers};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct KeyCombination {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl KeyCombination {
    pub fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    pub fn get_id(&self) -> usize {
        (self.key as usize) * 10usize.pow(Modifiers::BIT_COUNT as u32) + self.modifiers.get_id()
    }
}

impl Display for KeyCombination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.key.to_string();

        let modifier_s = [
            (self.modifiers.lalt, "LAlt"),
            (self.modifiers.ralt, "RAlt"),
            (self.modifiers.shift, "Shift"),
            (self.modifiers.win, "Win"),
            (self.modifiers.ctrl, "Ctrl"),
        ]
        .iter()
        .filter(|(x, _)| *x)
        .map(|(_, x)| x)
        .join("+");

        if !modifier_s.is_empty() {
            s = format!("{}+{}", modifier_s, s);
        }

        write!(f, "{}", s)
    }
}

impl FromStr for KeyCombination {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s
            .split('+')
            .map(|x| x.trim().to_ascii_uppercase())
            .collect();

        Ok(match parts.as_slice() {
            [key] => Self {
                key: Key::from_str(key)?,
                modifiers: Modifiers::default(),
            },
            [raw_modifiers @ .., key] => {
                let mut modifiers = Modifiers::default();

                for raw_modifier in raw_modifiers {
                    match raw_modifier.as_str() {
                        "CTRL" => modifiers.ctrl = true,
                        "SHIFT" => modifiers.shift = true,
                        "LALT" | "ALT" => modifiers.lalt = true,
                        "RALT" => modifiers.ralt = true,
                        "MOD" | "WIN" => modifiers.win = true,
                        m => return Err(format!("Unknown modifier '{}'", m)),
                    }
                }

                Self {
                    key: Key::from_str(key)?,
                    modifiers,
                }
            }
            [] => {
                return Err(String::from(
                    "An empty string is not a valid key combination",
                ))
            }
        })
    }
}
