use std::sync::Arc;

use crate::{
    config::Config,
    event::Event,
    graph::GraphNode,
    key_combination::KeyCombination,
    keybinding::KeybindingMode,
    keybinding_event_loop::KeybindingEventLoop,
    lua::LuaRuntime,
    platform::{Api, NativeApi, NativeDisplay, NativeWindow, Window},
    session,
    state::State,
};
use log::info;
use mlua::FromLua;
pub use window::WindowAction;
pub use workspace::WorkspaceAction;

mod window;
mod workspace;

macro_rules! action_fn {
    ($ident: ident, $($ty:ty),*) => {
        #[derive(Clone)]
        pub struct $ident(pub Arc<dyn Fn($($ty),*) -> () + Sync + Send>);

        impl $ident {
            pub fn new(f: impl Fn($($ty),*) -> () + Sync + Send + 'static) -> Self {
                Self(Arc::new(f))
            }
        }

        impl std::fmt::Debug for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!($ident))
            }
        }
    }
}

action_fn!(UpdateConfigActionFn, &mut Config);
action_fn!(ExecuteLuaActionFn, mlua::Result<String>);

#[derive(Debug, Clone)]
pub enum Action {
    SaveSession,
    LoadSession,
    ShowTaskbars,
    HideTaskbars,
    Window(WindowAction),
    Workspace(WorkspaceAction),
    UpdateConfig {
        key: String,
        update_fn: UpdateConfigActionFn,
    },
    CreateKeybinding {
        mode: KeybindingMode,
        key_combination: KeyCombination,
    },
    RemoveKeybinding {
        key: String,
    },
    ExecuteLua {
        code: String,
        capture_stdout: bool,
        cb: ExecuteLuaActionFn,
    },
}

impl Action {
    pub fn handle(self, state: &State, rt: &LuaRuntime) {
        match self {
            Action::Window(action) => match action {
                WindowAction::Focus(win_id) => {
                    let win = Window::new(win_id);
                    win.focus();
                }
                WindowAction::Close(maybe_win_id) => {
                    let win_id =
                        maybe_win_id.unwrap_or_else(|| Api::get_foreground_window().get_id());

                    Window::new(win_id).close();
                }
                WindowAction::Manage(maybe_id) => {
                    let win = maybe_id
                        .map(|id| Window::new(id))
                        .unwrap_or_else(|| Api::get_foreground_window());

                    state.with_focused_wm_mut(|wm| {
                        let workspace = wm.get_focused_workspace_mut();

                        if win.exists() && !workspace.has_window(win.get_id()) {
                            info!("'{}' managed", win.get_title());

                            wm.manage(&rt, &state.config.read(), win);
                        }
                    });
                }
                WindowAction::Unmanage(maybe_id) => state.with_focused_wm_mut(|wm| {
                    let workspace = wm.get_focused_workspace();
                    let win = maybe_id
                        .map(|id| Window::new(id))
                        .unwrap_or_else(|| Api::get_foreground_window());

                    if workspace.has_window(win.get_id()) {
                        info!("'{}' unmanaged", win.get_title());

                        wm.unmanage(&rt, &state.config.read(), win.get_id());
                    }
                }),
            },
            Action::Workspace(action) => match action {
                WorkspaceAction::Focus(maybe_id, dir) => state.with_focused_wm_mut(|wm| {
                    let workspace = wm.get_focused_workspace_mut();
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
                WorkspaceAction::Swap(maybe_id, dir) => state.with_focused_wm_mut(|wm| {
                    wm.swap_in_direction(&rt, &state.config.read(), None, dir);
                }),
            },
            Action::SaveSession => {
                session::save_session(&state.wms.read()[0].read().workspaces);
                info!("Saved session!");
            }
            Action::LoadSession => state.with_focused_wm_mut(|wm| {
                wm.workspaces = session::load_session(state.tx.clone()).unwrap();
                info!("Loaded session!");

                let mut windows = Vec::new();

                for ws in &wm.workspaces {
                    for node in ws.graph.nodes.values() {
                        if let GraphNode::Window(win_id) = node {
                            windows.push(Window::new(*win_id));
                        }
                    }
                }

                for window in windows {
                    wm.manage(&rt, &state.config.read(), window);
                }

                wm.render(&state.config.read());
            }),
            Action::ShowTaskbars => {
                let wms = state.wms.read();

                for wm in wms.iter() {
                    let mut wm = wm.write();
                    wm.display.show_taskbar();
                    wm.cleanup();
                }
            }
            Action::HideTaskbars => {
                let wms = state.wms.read();

                for wm in wms.iter() {
                    let mut wm = wm.write();
                    wm.display.hide_taskbar();
                    wm.cleanup();
                }
            }
            Action::UpdateConfig { key, update_fn } => {
                update_fn.0(&mut state.config.write());
                info!("Updated config property: {:#?}", key);
            }
            Action::ExecuteLua {
                code,
                capture_stdout,
                cb,
            } => {
                if capture_stdout {
                    rt.eval(
                        r#"
                            _G.__stdout_buf = ""
                            _G.__old_print = print
                            _G.print = function(...)
                                if _G.__stdout_buf ~= "" then
                                    _G.__stdout_buf = _G.__stdout_buf .. "\n"
                                end
                                local outputs = {}
                                for _,x in ipairs({...}) do
                                    table.insert(outputs, tostring(x))
                                end
                                local output = table.concat(outputs, "\t")
                                _G.__stdout_buf = _G.__stdout_buf .. output
                            end
                                    "#,
                    )
                    .unwrap();

                    let code_res = rt.eval(&code);

                    let stdout_buf =
                        String::from_lua(rt.eval("_G.__stdout_buf").unwrap(), rt.lua).unwrap();

                    cb.0(code_res.map(move |x| {
                        if stdout_buf.is_empty() {
                            format!("{:?}", x)
                        } else {
                            format!("{}\n{:?}", stdout_buf, x)
                        }
                    }));

                    rt.eval(
                        r#"
                            _G.print = _G.__old_print
                            _G.__stdout_buf = nil
                            _G.__old_print = nil
                                    "#,
                    )
                    .unwrap();
                } else {
                    cb.0(rt.eval(&code).map(|x| format!("{:?}", x)));
                }
            }
            Action::CreateKeybinding {
                mode,
                key_combination,
            } => {
                KeybindingEventLoop::add_keybinding(key_combination.get_id());
                info!("Created {:?} keybinding: {}", mode, key_combination);
            }
            Action::RemoveKeybinding { key } => {
                // KeybindingEventLoop::remove_keybinding(key_combination.get_id());
                info!("Removed keybinding: {}", key);
            }
        }
    }
}
