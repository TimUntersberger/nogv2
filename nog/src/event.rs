use std::sync::Arc;

use crate::action::Action;
use crate::key_combination::KeyCombination;
use crate::lua::LuaRuntime;
use crate::platform::Window;
use crate::state::State;
use crate::window_event_loop::WindowEvent;

#[derive(Clone)]
pub struct DeferedFunction(pub Arc<dyn Fn(&LuaRuntime, State) + Sync + Send>);

impl DeferedFunction {
    pub fn new(f: impl Fn(&LuaRuntime, State) + Sync + Send + 'static) -> Self {
        Self(Arc::new(f))
    }
}

impl std::fmt::Debug for DeferedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!($ident))
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    RenderGraph,
    ShowMenu,
    Exit,
    RenderBarLayout,
    Defered(DeferedFunction),
    Window(WindowEvent<Window>),
    Keybinding(KeyCombination),
    Action(Action),
}
