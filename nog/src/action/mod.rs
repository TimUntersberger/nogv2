use std::{mem, str::FromStr, sync::Arc};

use crate::{
    bar::Bar,
    config::{Config, ConfigProperty},
    event::Event,
    graph::GraphNode,
    key::Key,
    key_combination::KeyCombination,
    keybinding::{Keybinding, KeybindingMode},
    keybinding_event_loop::KeybindingEventLoop,
    lua::LuaRuntime,
    modifiers::Modifiers,
    notification::{Notification, NotificationManager},
    platform::{Api, NativeApi, NativeWindow, Window, WindowId},
    session,
    state::State,
    workspace::WorkspaceId,
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
    Launch(String),
    SaveSession(String),
    LoadSession(String),
    ShowTaskbars,
    HideTaskbars,
    ShowBars,
    HideBars,
    Awake,
    Hibernate,
    CreateNotification(Notification),
    MoveWindowToWorkspace(Option<WindowId>, WorkspaceId),
    SimulateKeyPress {
        key: Key,
        modifiers: Modifiers,
    },
    Window(WindowAction),
    Workspace(WorkspaceAction),
    UpdateConfig(ConfigProperty),
    /// When this event is received the new callback is already in the named registry of lua
    CreateKeybinding {
        mode: KeybindingMode,
        key_combination: KeyCombination,
    },
    RemoveKeybinding {
        key: String,
    },
    ExecuteLua {
        code: String,
        print_type: bool,
        capture_stdout: bool,
        cb: ExecuteLuaActionFn,
    },
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Action::CreateNotification(n) => String::from("Create notification"),
                Action::Launch(path) => format!("Launch '{}'", path),
                Action::SaveSession(name) => format!("Save session as '{}'", name),
                Action::LoadSession(name) => format!("Load session '{}'", name),
                Action::ShowTaskbars => format!("Show taskbars"),
                Action::HideTaskbars => format!("Hide taskbars"),
                Action::ShowBars => format!("Show bars"),
                Action::HideBars => format!("Hide bars"),
                Action::Awake => format!("Awake"),
                Action::Hibernate => format!("Hibernate"),
                Action::MoveWindowToWorkspace(window, workspace) =>
                    format!("Move Window({:?}) to Workspace({:?})", window, workspace),
                Action::SimulateKeyPress { key, modifiers } => format!(
                    "Simulate '{}'",
                    KeyCombination::new(key.clone(), modifiers.clone())
                ),
                Action::Window(inner) => format!("{}", inner),
                Action::Workspace(inner) => format!("{}", inner),
                Action::UpdateConfig(prop) => format!("config.{} updated", prop.get_name()),
                Action::CreateKeybinding {
                    mode,
                    key_combination,
                } => format!("{:?} Keybinding('{}') created", mode, key_combination),
                Action::RemoveKeybinding { key } => format!("Keybinding('{}') removed", key),
                Action::ExecuteLua {
                    code,
                    print_type,
                    capture_stdout,
                    cb,
                } => format!(
                    "Executing lua string (Stdout: {}, Type: {})",
                    capture_stdout, print_type
                ),
            }
        )
    }
}

impl Action {
    pub fn handle(
        self,
        state: &State,
        rt: &LuaRuntime,
        notification_manager: &mut NotificationManager,
    ) {
        match &self {
            Action::Window(_) | Action::Workspace(_) => {}
            _ => log::trace!("{}", &self),
        }
        match self {
            Action::Launch(path) => Api::launch(path),
            Action::MoveWindowToWorkspace(win_id, ws_id) => {
                let win_id = win_id.or_else(|| {
                    state.with_focused_dsp(|dsp| {
                        dsp.wm
                            .get_focused_workspace()
                            .get_focused_win()
                            .map(|x| x.get_id())
                    })
                });

                if let Some(win_id) = win_id {
                    WindowAction::Unmanage(Some(win_id)).handle(state, rt);
                    WindowAction::Manage(Some(ws_id), Some(win_id)).handle(state, rt);
                }
            }
            Action::Awake => {
                info!("Awoke!");

                if state.config.read().display_app_bar {
                    state.tx.send(Event::Action(Action::ShowBars)).unwrap();
                }

                if state.config.read().remove_task_bar {
                    state.tx.send(Event::Action(Action::HideTaskbars)).unwrap();
                }

                for kb in state.keybindings.read().iter() {
                    if kb.mode != KeybindingMode::Global {
                        KeybindingEventLoop::add_keybinding(kb.get_id());
                    }
                }

                state.with_focused_dsp_mut(|dsp| dsp.wm.change_workspace(&rt, WorkspaceId(1)));
                state.awake();
            }
            Action::SimulateKeyPress { key, modifiers } => {
                Api::simulate_key_press(key, modifiers);
            }
            Action::Hibernate => {
                state.tx.send(Event::Action(Action::HideBars)).unwrap();
                state.tx.send(Event::Action(Action::ShowTaskbars)).unwrap();
                for d in state.displays.write().iter_mut() {
                    d.wm.cleanup();
                }

                for kb in state.keybindings.read().iter() {
                    if kb.mode != KeybindingMode::Global {
                        KeybindingEventLoop::remove_keybinding(kb.get_id());
                    }
                }

                state.hibernate();
            }
            Action::Window(action) => action.handle(state, rt),
            Action::Workspace(action) => action.handle(state, rt),
            Action::SaveSession(name) => {
                session::save_session(&name, &state.displays.read()[0].wm.workspaces);
            }
            Action::LoadSession(name) => state.with_focused_dsp_mut(|d| {
                d.wm.workspaces = session::load_session(&name).unwrap();
                let area = d.get_render_area(&state.config.read());

                let mut ws_windows = Vec::new();

                for ws in &d.wm.workspaces {
                    for node in ws.graph.nodes.values() {
                        if let GraphNode::Window(win_id) = node {
                            ws_windows.push((ws.id, Window::new(*win_id)));
                        }
                    }
                }

                for (ws_id, window) in ws_windows {
                    d.wm.manage(rt, &state.config.read(), Some(ws_id), area, window)
                        .unwrap();
                }

                d.wm.render(&state.config.read(), area);
            }),
            Action::ShowTaskbars => {
                for d in state.displays.write().iter_mut() {
                    d.show_taskbar();
                }
            }
            Action::HideTaskbars => {
                for d in state.displays.write().iter_mut() {
                    d.hide_taskbar();
                }
            }
            Action::UpdateConfig(prop) => {
                if state.is_awake() {
                    let event = match prop {
                        ConfigProperty::FontSize(_)
                        | ConfigProperty::FontName(_)
                        | ConfigProperty::BarHeight(_) => {
                            Some(Event::BatchAction(vec![Action::HideBars, Action::ShowBars]))
                        }
                        ConfigProperty::OuterGap(_) | ConfigProperty::InnerGap(_) => {
                            Some(Event::RenderGraph)
                        }
                        ConfigProperty::RemoveTaskBar(old_value) => {
                            match old_value != state.config.read().remove_task_bar {
                                true => Some(match old_value {
                                    true => Event::Action(Action::ShowTaskbars),
                                    false => Event::Action(Action::HideTaskbars),
                                }),
                                false => None,
                            }
                        }
                        ConfigProperty::DisplayAppBar(old_value) => {
                            match old_value != state.config.read().display_app_bar {
                                true => Some(match old_value {
                                    true => Event::Action(Action::HideBars),
                                    false => Event::Action(Action::ShowBars),
                                }),
                                false => None,
                            }
                        }
                        ConfigProperty::LightTheme(_)
                        | ConfigProperty::Color(_)
                        | ConfigProperty::MultiMonitor(_)
                        | ConfigProperty::RemoveDecorations(_)
                        | ConfigProperty::IgnoreFullscreenActions(_) => None,
                    };

                    if let Some(event) = event {
                        state.tx.send(event).unwrap();
                    }
                }
            }
            Action::ExecuteLua {
                code,
                print_type,
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
                            if print_type {
                                format!("{:?}", x)
                            } else {
                                String::from("")
                            }
                        } else {
                            format!(
                                "{}{}",
                                stdout_buf,
                                if print_type {
                                    format!("{:?}\n", x)
                                } else {
                                    String::from("")
                                }
                            )
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
                state.keybindings.write().push(Keybinding {
                    mode,
                    key_combination,
                });
            }
            Action::RemoveKeybinding { key } => {
                let kc_id = KeyCombination::from_str(&key).unwrap().get_id();

                let kb_idx = state
                    .keybindings
                    .read()
                    .iter()
                    .enumerate()
                    .find(|(_, kb)| kb.get_id() == kc_id)
                    .map(|(idx, _)| idx)
                    .unwrap();

                state.keybindings.write().remove(kb_idx);

                KeybindingEventLoop::remove_keybinding(kc_id);
            }
            Action::CreateNotification(n) => {
                notification_manager.push(n);
            }
            Action::ShowBars => {
                for d in state.displays.write().iter_mut() {
                    if d.bar.is_none() {
                        d.bar = Some(Bar::new().unwrap());
                    }
                }
            }
            Action::HideBars => {
                for d in state.displays.write().iter_mut() {
                    if let Some(mut bar) = mem::take(&mut d.bar) {
                        bar.close();
                    }
                }
            }
        }
    }
}
