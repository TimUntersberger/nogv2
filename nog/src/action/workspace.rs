use std::fmt::Display;

use crate::{
    direction::Direction,
    event::Event,
    lua::LuaRuntime,
    platform::NativeWindow,
    state::State,
    workspace::{Workspace, WorkspaceId, WorkspaceState},
};

use super::{Action, WindowAction};

#[derive(Debug, Clone)]
pub enum WorkspaceAction {
    Change(WorkspaceId),
    SetFullscreen(Option<WorkspaceId>, bool),
    Focus(Option<WorkspaceId>, Direction),
    Swap(Option<WorkspaceId>, Direction),
}

impl Display for WorkspaceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WorkspaceAction::Change(id) => format!("WorkspaceAction::Change({})", id.0),
                WorkspaceAction::SetFullscreen(id, value) =>
                    format!("WorkspaceAction::SetFullscreen({:?}, {})", id, value),
                WorkspaceAction::Focus(id, direction) =>
                    format!("WorkspaceAction::Focus({:?}, {})", id, direction),
                WorkspaceAction::Swap(id, direction) =>
                    format!("WorkspaceAction::Swap({:?}, {})", id, direction),
            }
        )
    }
}

impl WorkspaceAction {
    pub fn handle(self, state: &State, rt: &LuaRuntime) {
        log::trace!("{}", &self);
        match self {
            WorkspaceAction::SetFullscreen(maybe_id, value) => {
                state.with_ws_mut(WorkspaceId(1), |ws| {
                    ws.state = if value {
                        WorkspaceState::Fullscreen
                    } else {
                        WorkspaceState::Normal
                    }
                });
            }
            WorkspaceAction::Focus(maybe_id, dir) => state.with_focused_dsp_mut(|d| {
                let workspace = d.wm.get_focused_workspace_mut();
                if let Some(id) = workspace.focus_in_direction(dir) {
                    let win_id = workspace
                        .graph
                        .get_node(id)
                        .expect("The returned node has to exist")
                        .try_get_window_id()
                        .expect("The focused node has to be a window node");

                    state
                        .tx
                        .send(Event::Action(Action::Window(WindowAction::Focus(win_id))))
                        .unwrap();
                }
            }),
            WorkspaceAction::Swap(maybe_id, dir) => state.with_focused_dsp_mut(|d| {
                let area = d.get_render_area(&state.config.read());
                d.wm.swap_in_direction(rt, &state.config.read(), area, None, dir)
                    .unwrap();
            }),
            WorkspaceAction::Change(id) => {
                // There are two cases to consider:
                //  * The new workspace doesn't exist yet
                //  * The new workspace already exists

                // `res` is an Option<Option<Window>>
                //
                // The first option represents whether the workspace already exists
                // and the second option whether the workspace has a focused window
                let res = state.with_ws(id, |ws| ws.get_focused_win());

                match res {
                    Some(maybe_focused_win) => match maybe_focused_win {
                        Some(win) => {
                            state.with_dsp_containing_ws_mut(id, |dsp| {
                                if dsp.wm.focus_window(win.get_id()) {
                                    win.focus();
                                }
                            });
                        }
                        None => {}
                    },
                    None => {
                        state.with_focused_dsp_mut(|dsp| dsp.wm.change_workspace(id));
                    }
                };
            }
        }
    }
}
