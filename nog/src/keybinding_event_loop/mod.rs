pub mod win;
use crate::key_combination::KeyCombination;
pub use win::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputEvent {
    KeyDown(KeyCombination),
    KeyUp(KeyCombination),
}

pub struct KeybindingEventLoop;
