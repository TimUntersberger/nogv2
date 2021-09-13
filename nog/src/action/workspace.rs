use crate::{
    direction::Direction, event::Event, lua::LuaRuntime, platform::NativeMonitor, state::State,
    workspace::WorkspaceId,
};

use super::{Action, WindowAction};

#[derive(Debug, Clone)]
pub enum WorkspaceAction {
    Focus(Option<WorkspaceId>, Direction),
    Swap(Option<WorkspaceId>, Direction),
}

impl WorkspaceAction {
    pub fn handle(self, state: &State, rt: &LuaRuntime) {
        match self {
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
                let area = d.monitor.get_work_area();
                d.wm.swap_in_direction(rt, &state.config.read(), area, None, dir)
                    .unwrap();
            }),
        }
    }
}
