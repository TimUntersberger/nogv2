use std::sync::Arc;

use crate::{
    bar::Bar,
    config::Config,
    graph::GraphNode,
    key_combination::KeyCombination,
    keybinding::KeybindingMode,
    keybinding_event_loop::KeybindingEventLoop,
    lua::LuaRuntime,
    platform::{NativeMonitor, NativeWindow, Window},
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
        pub struct $ident(pub Arc<dyn Fn($($ty),*) + Sync + Send>);

        impl $ident {
            pub fn new(f: impl Fn($($ty),*) + Sync + Send + 'static) -> Self {
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
    ShowBars,
    HideBars,
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
            Action::Window(action) => action.handle(state, rt),
            Action::Workspace(action) => action.handle(state, rt),
            Action::SaveSession => {
                session::save_session(&state.displays.read()[0].wm.workspaces);
                info!("Saved session!");
            }
            Action::LoadSession => state.with_focused_dsp_mut(|d| {
                d.wm.workspaces = session::load_session().unwrap();
                let area = d.monitor.get_work_area();
                info!("Loaded session!");

                let mut windows = Vec::new();

                for ws in &d.wm.workspaces {
                    for node in ws.graph.nodes.values() {
                        if let GraphNode::Window(win_id) = node {
                            windows.push(Window::new(*win_id));
                        }
                    }
                }

                for window in windows {
                    d.wm.manage(rt, &state.config.read(), area, window).unwrap();
                }

                d.wm.render(&state.config.read(), area);
            }),
            Action::ShowTaskbars => {
                for d in state.displays.write().iter_mut() {
                    d.show_taskbar();
                    d.wm.cleanup();
                }
            }
            Action::HideTaskbars => {
                for d in state.displays.write().iter_mut() {
                    d.hide_taskbar();
                    d.wm.cleanup();
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
            Action::ShowBars => {
                for d in state.displays.write().iter_mut() {
                    if d.bar.is_none() {
                        d.bar = Some(Bar::new().unwrap());
                    }
                }
            }
            Action::HideBars => todo!(),
        }
    }
}
