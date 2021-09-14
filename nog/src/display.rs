use crate::bar::Bar;
use crate::platform::{Monitor, NativeWindow, Window};
use crate::window_manager::WindowManager;

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayId(pub String);

#[derive(Debug)]
pub struct Display {
    pub id: DisplayId,
    pub taskbar_win: Window,
    pub bar: Option<Bar>,
    pub wm: WindowManager,
    pub monitor: Monitor,
}

impl Display {
    pub fn show_taskbar(&self) {
        self.taskbar_win.show();
    }

    pub fn hide_taskbar(&self) {
        self.taskbar_win.hide();
    }
}
