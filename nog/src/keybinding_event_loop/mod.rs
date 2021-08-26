pub mod win;
use itertools::*;
pub use win::*;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Modifiers {
    pub lalt: bool,
    pub ralt: bool,
    pub shift: bool,
    pub win: bool,
    pub ctrl: bool,
}

impl Modifiers {
    /// how many fields does modifiers have
    pub const BIT_COUNT: u8 = 5;
    pub fn get_id(&self) -> usize {
        [
            self.lalt as u8,
            self.ralt as u8,
            self.shift as u8,
            self.win as u8,
            self.ctrl as u8,
        ]
        .iter()
        .enumerate()
        .fold(0usize, |acc, (idx, x)| {
            acc + *x as usize * 10usize.pow(idx as u32)
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputEvent {
    KeyDown(Keybinding),
    KeyUp(Keybinding),
}

fn key_code_to_string(key_code: usize) -> String {
    match key_code {
        13 => "Enter".to_string(),
        27 => "Escape".to_string(),
        k => (key_code as u8 as char).to_string(),
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Keybinding {
    pub key_code: usize,
    pub modifiers: Modifiers,
}

impl Keybinding {
    pub fn get_id(&self) -> usize {
        self.key_code * 10usize.pow(Modifiers::BIT_COUNT as u32) + self.modifiers.get_id()
    }

    pub fn to_string(&self) -> String {
        let mut s = key_code_to_string(self.key_code);

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

        s
    }
}

pub struct KeybindingEventLoop;
