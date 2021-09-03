use std::sync::Arc;

use crate::config::Config;
use crate::direction::Direction;
use crate::key_combination::KeyCombination;
use crate::keybinding::{Keybinding, KeybindingMode};
use crate::platform::{Window, WindowId};
use crate::window_event_loop::WindowEvent;
use crate::workspace::WorkspaceId;

macro_rules! action_fn {
    ($ident: ident, $($ty:ty),*) => {
        #[derive(Clone)]
        pub struct $ident(pub Arc<dyn Fn($($ty),*) -> () + Sync + Send>);

        impl $ident {
            pub fn new(f: impl Fn($($ty),*) -> () + Sync + Send + 'static) -> Self {
                Self(Arc::new(f))
            }
        }

        impl std::fmt::Debug for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!($ident))
            }
        }
    }
}

action_fn!(UpdateConfigActionFn, &mut Config);
action_fn!(ExecuteLuaActionFn, mlua::Result<String>);

#[derive(Debug, Clone)]
pub enum WorkspaceAction {
    Focus(Option<WorkspaceId>, Direction),
    Swap(Option<WorkspaceId>, Direction),
}

#[derive(Debug, Clone)]
pub enum WindowAction {
    Focus(WindowId),
    Manage(Option<WindowId>),
    Unmanage(Option<WindowId>),
    Close(Option<WindowId>),
}

#[derive(Debug, Clone)]
pub enum Action {
    SaveSession,
    LoadSession,
    ShowTaskbars,
    HideTaskbars,
    Window(WindowAction),
    Workspace(WorkspaceAction),
    UpdateConfig {
        key: String,
        update_fn: UpdateConfigActionFn,
    },
    CreateKeybinding {
        mode: KeybindingMode,
        key_combination: KeyCombination,
    },
    RemoveKeybinding {
        key: String,
    },
    ExecuteLua {
        code: String,
        capture_stdout: bool,
        cb: ExecuteLuaActionFn,
    },
}

#[derive(Debug, Clone)]
pub enum Event {
    RenderGraph,
    Exit,
    Window(WindowEvent<Window>),
    Keybinding(Keybinding),
    Action(Action),
}
