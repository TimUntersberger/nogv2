use crate::keybinding_event_loop::Keybinding;
use crate::window_event_loop::WindowEvent;
use crate::platform::Window;

#[derive(Debug, Clone)]
pub enum Event {
    Window(WindowEvent<Window>),
    Keybinding(Keybinding)
}
