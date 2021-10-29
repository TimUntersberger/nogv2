use crate::action::Action;
use crate::key_combination::KeyCombination;
use crate::keybinding::Keybinding;
use crate::platform::Window;
use crate::window_event_loop::WindowEvent;

#[derive(Debug, Clone)]
pub enum Event {
    RenderGraph,
    ShowMenu,
    Exit,
    RenderBarLayout,
    Window(WindowEvent<Window>),
    Keybinding(KeyCombination),
    Action(Action),
}
