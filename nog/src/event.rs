use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::keybinding_event_loop::Keybinding;
use crate::keybinding::KeybindingMode;
use crate::platform::Window;
use crate::window_event_loop::WindowEvent;

#[derive(Clone)]
pub struct ActionFn(pub Arc<dyn Fn(&mut Config) -> () + Sync + Send>);

impl ActionFn {
    pub fn new(f: impl Fn(&mut Config) -> () + Sync + Send + 'static) -> Self {
        Self(Arc::new(f))
    }
}

impl std::fmt::Debug for ActionFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ActionFn")
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    UpdateConfig {
        key: String,
        update_fn: ActionFn
    },
    CreateKeybinding {
        mode: KeybindingMode,
        key: String
    },
    RemoveKeybinding {
        key: String
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Window(WindowEvent<Window>),
    Keybinding(Keybinding),
    Action(Action),
}
