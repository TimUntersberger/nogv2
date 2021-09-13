use log::info;

use crate::{
    lua::LuaRuntime,
    platform::{Api, NativeApi, NativeWindow, NativeMonitor, Window, WindowId},
    state::State,
};

#[derive(Debug, Clone)]
pub enum WindowAction {
    Focus(WindowId),
    Manage(Option<WindowId>),
    Unmanage(Option<WindowId>),
    Close(Option<WindowId>),
}

impl WindowAction {
    pub fn handle(self, state: &State, rt: &LuaRuntime) {
        match self {
            WindowAction::Focus(win_id) => {
                let win = Window::new(win_id);
                win.focus();
            }
            WindowAction::Close(maybe_win_id) => {
                let win_id = maybe_win_id.unwrap_or_else(|| Api::get_foreground_window().get_id());

                Window::new(win_id).close();
            }
            WindowAction::Manage(maybe_id) => {
                let win = maybe_id
                    .map(|id| Window::new(id))
                    .unwrap_or_else(|| Api::get_foreground_window());

                state.with_focused_dsp_mut(|d| {
                    let workspace = d.wm.get_focused_workspace_mut();
                    let area = d.monitor.get_work_area();

                    if win.exists() && !workspace.has_window(win.get_id()) {
                        info!("'{}' managed", win.get_title());

                        d.wm.manage(&rt, &state.config.read(), area, win);
                    }
                });
            }
            WindowAction::Unmanage(maybe_id) => state.with_focused_dsp_mut(|d| {
                let workspace = d.wm.get_focused_workspace();
                let area = d.monitor.get_work_area();
                let win = maybe_id
                    .map(|id| Window::new(id))
                    .unwrap_or_else(|| Api::get_foreground_window());

                if workspace.has_window(win.get_id()) {
                    info!("'{}' unmanaged", win.get_title());

                    d.wm.unmanage(&rt, &state.config.read(), area, win.get_id());
                }
            }),
        }
    }
}
