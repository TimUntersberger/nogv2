use crate::bar::Bar;
use crate::platform::{Monitor, Window};
use crate::window_manager::WindowManager;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DisplayId(pub usize);

pub struct Display {
    pub id: DisplayId,
    pub taskbar_win: Window,
    pub bar: Option<Bar>,
    pub wm: WindowManager,
    pub monitor: Monitor,
}

impl Display {
    pub fn show_taskbar(&self) {
        todo!()
    }

    pub fn hide_taskbar(&self) {
        todo!()
    }
}
