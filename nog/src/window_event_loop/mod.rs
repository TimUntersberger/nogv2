use crate::platform::NativeWindow;

mod win;

#[derive(Debug, Clone)]
pub enum WindowEventKind {
    Created,
    Deleted,
    Minimized,
    FocusChanged,
}

#[derive(Debug, Clone)]
pub struct WindowEvent<TWin: NativeWindow> {
    pub window: TWin,
    pub kind: WindowEventKind,
}

pub struct WindowEventLoop;
