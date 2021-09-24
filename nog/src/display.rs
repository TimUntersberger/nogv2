use crate::bar::Bar;
use crate::config::Config;
use crate::platform::{Area, Monitor, NativeMonitor, NativeWindow, Window};
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

    /// The area on the display where the windows should get rendered to
    pub fn get_render_area(&self, config: &Config) -> Area {
        let mut area = self.monitor.get_work_area();

        if config.display_app_bar {
            area.pos.y += config.bar_height as isize;
            area.size.height -= config.bar_height as usize;
        }

        if config.remove_task_bar {
            // TODO: revisit this.
            //
            // This is only temporary, we should fetch the taskbar size instead of hardcoding it.
            area.size.height += 40;
        }

        area
    }
}
