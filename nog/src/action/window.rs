use log::info;

use crate::{
    event::Event,
    lua::{self, LuaEvent, LuaRuntime},
    platform::{Api, NativeApi, NativeWindow, Window, WindowId},
    state::State,
    window_event_loop::{WindowEvent, WindowEventKind},
    workspace::WorkspaceId,
};

#[derive(Debug, Clone)]
pub enum WindowAction {
    Focus(WindowId),
    Manage(Option<WorkspaceId>, Option<WindowId>),
    Unmanage(Option<WindowId>),
    Close(Option<WindowId>),
    Minimize(Option<WindowId>),
}

impl std::fmt::Display for WindowAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WindowAction::Focus(id) => format!("Focusing Window({})", id.0),
                WindowAction::Manage(ws_id, id) =>
                    format!("Managing Window({:?}) in Workspace({:?})", id, ws_id),
                WindowAction::Unmanage(id) => format!("Unmanaging Window({:?})", id),
                WindowAction::Close(id) => format!("Closing Window({:?})", id),
                WindowAction::Minimize(id) => format!("Minimizing Window({:?})", id),
            }
        )
    }
}

impl WindowAction {
    pub fn handle(self, state: &State, rt: &LuaRuntime) {
        log::trace!("{}", self);
        match self {
            WindowAction::Focus(win_id) => {
                let win = Window::new(win_id);
                win.focus();
            }
            WindowAction::Close(maybe_win_id) => {
                let win_id = maybe_win_id.unwrap_or_else(|| Api::get_foreground_window().get_id());
                let win = Window::new(win_id);

                info!("Closing '{}'", win.get_title());

                win.close();

                state
                    .tx
                    .send(Event::Window(WindowEvent {
                        kind: WindowEventKind::Deleted,
                        window: win,
                    }))
                    .unwrap();
            }
            WindowAction::Minimize(maybe_win_id) => {
                let win_id = maybe_win_id.unwrap_or_else(|| Api::get_foreground_window().get_id());
                let win = Window::new(win_id);

                info!("Minimizing '{}'", win.get_title());

                win.minimize();

                state
                    .tx
                    .send(Event::Window(WindowEvent {
                        // TODO: Right now Minimized is not getting handled. Once we handle the
                        // minimized event correctly change from deleted to minimized.
                        kind: WindowEventKind::Deleted,
                        window: win,
                    }))
                    .unwrap();
            }
            WindowAction::Manage(ws_id, maybe_win_id) => {
                let win = maybe_win_id
                    .map(Window::new)
                    .unwrap_or_else(Api::get_foreground_window);

                let ws_id = ws_id
                    .unwrap_or_else(|| state.with_focused_dsp(|dsp| dsp.wm.focused_workspace_id));

                let already_managed = state
                    .with_dsp_containing_win_mut(win.get_id(), |_| {})
                    .is_some();

                if already_managed {
                    return;
                }

                lua::emit_manage(
                    &rt,
                    LuaEvent::Manage {
                        manual: true,
                        ws_id: Some(ws_id),
                        win_id: win.get_id(),
                    },
                )
                .unwrap();

                state.create_workspace(state.get_focused_dsp_id(), ws_id);

                state.with_dsp_containing_ws_mut(ws_id, |d| {
                    d.wm.change_workspace(ws_id);

                    let area = d.get_render_area(&state.config.read());
                    let workspace = d.wm.get_ws_by_id(ws_id).unwrap();

                    if win.exists() && !workspace.has_window(win.get_id()) {
                        info!("'{}' managed", win.get_title());

                        let ws_id = workspace.id;

                        d.wm.manage(rt, &state.config.read(), Some(ws_id), area, win)
                            .unwrap();
                    }
                });
            }
            WindowAction::Unmanage(maybe_id) => state.with_focused_dsp_mut(|d| {
                let workspace = d.wm.get_focused_workspace();
                let area = d.get_render_area(&state.config.read());

                let win = maybe_id
                    .map(Window::new)
                    .or_else(|| workspace.get_focused_win());

                if let Some(win) = win {
                    if workspace.has_window(win.get_id()) {
                        info!("'{}' unmanaged", win.get_title());

                        d.wm.unmanage(rt, &state.config.read(), area, win.get_id())
                            .unwrap();
                    }
                }
            }),
        }
    }
}
