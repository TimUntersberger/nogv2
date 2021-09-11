use crate::platform::WindowId;

#[derive(Debug, Clone)]
pub enum WindowAction {
    Focus(WindowId),
    Manage(Option<WindowId>),
    Unmanage(Option<WindowId>),
    Close(Option<WindowId>),
}
