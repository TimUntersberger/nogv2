pub mod win;
use crate::key_combination::KeyCombination;
pub use win::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputEvent {
    KeyDown(KeyCombination),
    KeyUp(KeyCombination),
}

fn key_code_to_string(key_code: usize) -> String {
    match key_code {
        13 => "Enter".to_string(),
        27 => "Escape".to_string(),
        k => (key_code as u8 as char).to_string(),
    }
}

pub struct KeybindingEventLoop;
