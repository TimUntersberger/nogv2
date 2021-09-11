use chrono::Duration;
use event::{Action, Event, WindowAction, WorkspaceAction};
use graph::GraphNodeId;
use keybinding_event_loop::KeybindingEventLoop;
use log::{error, info};
use lua::{graph_proxy::GraphProxy, LuaRuntime};
use mlua::FromLua;
use nog_protocol::{BarContent, BarItem, BarItemAlignment};
use platform::{Position, Size, Window, WindowId};
use rgb::RGB;
use server::Server;
use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Sender},
        Arc, RwLock,
    },
    thread,
};
use window_event_loop::WindowEventLoop;
use window_manager::WindowManager;
use workspace::Workspace;

use crate::{
    cleanup::{DisplayCleanup, WindowCleanup},
    config::Config,
    graph::{Graph, GraphNode, GraphNodeGroupKind},
    platform::{Api, Display, NativeApi, NativeDisplay, NativeWindow},
    window_event_loop::WindowEventKind,
};

/// Responsible for handling events like when a window is created, deleted, etc.
pub trait EventLoop {
    fn run(tx: Sender<Event>);
    fn stop();
    fn spawn(tx: Sender<Event>) {
        thread::spawn(move || {
            Self::run(tx);
        });
    }
}

mod cleanup;
mod config;
mod direction;
mod event;
mod graph;
mod key;
mod key_combination;
mod keybinding;
mod keybinding_event_loop;
mod logging;
mod lua;
mod modifiers;
mod paths;
mod display;
mod platform;
mod rgb;
mod server;
mod session;
mod window_event_loop;
mod window_manager;
mod workspace;

fn lua_value_to_bar_item<'a>(
    lua: &mlua::Lua,
    align: BarItemAlignment,
    value: mlua::Value<'a>,
    default_fg: RGB,
    default_bg: RGB,
) -> mlua::Result<BarItem> {
    Ok(match value {
        tbl @ mlua::Value::Table(..) => {
            let tbl = mlua::Table::from_lua(tbl, lua).unwrap();
            let text = String::from_lua(tbl.get(1).unwrap_or(mlua::Value::Nil), lua).unwrap();
            let fg = tbl.get::<&str, i32>("fg").map(RGB::from_hex).unwrap_or(default_fg);
            let bg = tbl.get::<&str, i32>("bg").map(RGB::from_hex).unwrap_or(default_bg);

            BarItem {
                text,
                alignment: align,
                fg: fg.0,
                bg: fg.0,
            }
        }
        value => {
            let text = lua
                .coerce_string(value)?
                .map(|s| s.to_str().unwrap().to_string())
                .unwrap_or(String::from("nil"));

            BarItem {
                text,
                alignment: align,
                fg: default_fg.0,
                bg: default_bg.0,
            }
        }
    })
}

fn main() {
    logging::init().expect("Failed to initialize logging");
    info!("Initialized logging");

    let (tx, rx) = channel::<Event>();
    let wm = Arc::new(RwLock::new(WindowManager::new(tx.clone())));
    let displays = Api::get_displays();
    let bar_content = Arc::new(RwLock::new(BarContent::default()));

    let bar_content_timer = {
        let timer = timer::Timer::new();
        let tx = tx.clone();
        (
            timer.schedule_repeating(Duration::milliseconds(100), move || {
                tx.send(Event::RenderBarLayout).unwrap();
            }),
            timer,
        )
    };

    {
        let tx = tx.clone();
        ctrlc::set_handler(move || tx.send(Event::Exit).unwrap());
    }

    let rt = match lua::init(tx.clone(), wm.clone()) {
        Ok(x) => x,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    // Run the config
    if let Err(e) = rt.eval("dofile(nog.config_path .. '/lua/config.lua')") {
        error!("Error when running config: {}", e);
    }

    let mut config = Config::default();

    if config.remove_task_bar {
        tx.send(Event::Action(Action::HideTaskbars)).unwrap();
    }

    // lua::repl::spawn(tx.clone());
    // info!("Repl started");

    Server::spawn(tx.clone(), bar_content.clone());
    info!("IPC Server started");

    WindowEventLoop::spawn(tx.clone());
    info!("Window event loop spawned");

    KeybindingEventLoop::spawn(tx.clone());
    info!("Keybinding event loop spawned");

    info!("Starting main event loop");
    while let Ok(event) = rx.recv() {
        match event {
            Event::Window(win_event) => match win_event.kind {
                WindowEventKind::FocusChanged => {
                    let mut wm = wm.write().unwrap();
                    let workspace = wm.get_focused_workspace_mut();
                    if workspace.focus_window(win_event.window.get_id()).is_ok() {
                        info!("Focused window with id {}", win_event.window.get_id());
                        win_event.window.focus();
                    }
                }
                WindowEventKind::Created => {
                    let win = win_event.window;
                    let size = win.get_size();

                    if size.width >= config.min_width && size.height >= config.min_height {
                        info!("'{}' created", win.get_title());

                        wm.write().unwrap().manage(&rt, &config, &displays[0], win);
                    }
                }
                WindowEventKind::Deleted => {
                    let win = win_event.window;
                    info!("'{}' deleted", win.get_title());

                    wm.write().unwrap().organize(
                        &rt,
                        &config,
                        &displays[0],
                        None,
                        String::from("deleted"),
                        win.get_id(),
                    );
                }
                WindowEventKind::Minimized => {
                    let win = win_event.window;
                    info!("'{}' minimized", win.get_title());

                    wm.write()
                        .unwrap()
                        .unmanage(&rt, &config, &displays[0], win.get_id());
                }
            },
            Event::RenderBarLayout => {
                let default_fg = RGB([1.0, 1.0, 1.0]);
                let default_bg = RGB([0.0, 0.0, 0.0]);

                macro_rules! convert_sections {
                    {$(($ident:ident, $s:expr)),*} => {
                        {
                            let mut result = Vec::new();
                            $(
                                //TODO: proper error handling instead of expecting values
                                for value in rt
                                    .rt
                                    .named_registry_value::<str, mlua::Table>($s)
                                    .expect(&format!("Registry value of {} bar layout section missing", $s))
                                    .sequence_values()
                                    .map(|v| mlua::Function::from_lua(v.unwrap(), &rt.rt)
                                            .expect("Has to be a function")
                                            .call::<(), mlua::Value>(())
                                            .expect("Cannot error")

                                    ) {
                                        match value {
                                            tbl @ mlua::Value::Table(..) => {
                                                let tbl = mlua::Table::from_lua(tbl, rt.rt).unwrap();
                                                for value in tbl.sequence_values() {
                                                    result.push(
                                                        lua_value_to_bar_item(
                                                            &rt.rt,
                                                            BarItemAlignment::$ident,
                                                            value.unwrap(),
                                                            default_fg,
                                                            default_bg
                                                        ).unwrap()
                                                    );
                                                }
                                            },
                                            value => {
                                                result.push(
                                                    lua_value_to_bar_item(
                                                        &rt.rt,
                                                        BarItemAlignment::$ident,
                                                        value,
                                                        default_fg,
                                                        default_bg
                                                    ).unwrap()
                                                );
                                            }
                                        };
                                    }
                            )*
                            result
                        }
                    }
                }

                let items = convert_sections! {
                    (Left, "left"),
                    (Center, "center"),
                    (Right, "right")
                };

                dbg!(&items);

                *bar_content.write().unwrap() = BarContent {
                    bg: [0.0, 0.0, 0.0],
                    items,
                };
            }
            Event::Keybinding(kb) => {
                info!("Received keybinding {}", kb.to_string());

                let cb = rt
                    .rt
                    .named_registry_value::<str, mlua::Function>(&kb.get_id().to_string())
                    .expect("Registry value of a keybinding somehow disappeared?");

                if let Err(e) = cb.call::<(), ()>(()) {
                    error!(
                        "{}",
                        match e {
                            mlua::Error::CallbackError { cause, .. } => cause.to_string(),
                            e => e.to_string(),
                        }
                    );
                }
            }
            Event::Action(action) => match action {
                Action::Window(action) => match action {
                    WindowAction::Focus(win_id) => {
                        let win = Window::new(win_id);
                        win.focus();
                    }
                    WindowAction::Close(maybe_win_id) => {
                        let wm = wm.read().unwrap();
                        let workspace = wm.get_focused_workspace();
                        let maybe_win_id = maybe_win_id.or_else(|| {
                            workspace
                                .get_focused_node()
                                .and_then(|n| n.try_get_window_id())
                        });

                        if let Some(id) = maybe_win_id {
                            Window::new(id).close();
                        }
                    }
                    WindowAction::Manage(maybe_id) => {
                        let mut wm = wm.write().unwrap();
                        let workspace = wm.get_focused_workspace_mut();
                        let win = maybe_id
                            .map(|id| Window::new(id))
                            .unwrap_or_else(|| Api::get_foreground_window());

                        if win.exists() && !workspace.has_window(win.get_id()) {
                            info!("'{}' managed", win.get_title());

                            wm.manage(&rt, &config, &displays[0], win);
                        }
                    }
                    WindowAction::Unmanage(maybe_id) => {
                        let mut wm = wm.write().unwrap();
                        let workspace = wm.get_focused_workspace();
                        let maybe_id = maybe_id.or(workspace
                            .get_focused_node()
                            .and_then(|x| x.try_get_window_id()));

                        if let Some(id) = maybe_id {
                            let win = Window::new(id);
                            if workspace.has_window(id) {
                                info!("'{}' unmanaged", win.get_title());

                                wm.unmanage(&rt, &config, &displays[0], id);
                            }
                        }
                    }
                },
                Action::Workspace(action) => match action {
                    WorkspaceAction::Focus(maybe_id, dir) => {
                        let mut wm = wm.write().unwrap();
                        let workspace = wm.get_focused_workspace_mut();
                        if let Some(id) = workspace.focus_in_direction(dir) {
                            let win_id = workspace
                                .graph
                                .get_node(id)
                                .expect("The returned node has to exist")
                                .try_get_window_id()
                                .expect("The focused node has to be a window node");

                            tx.send(Event::Action(Action::Window(WindowAction::Focus(win_id))))
                                .unwrap();
                        }
                    }
                    WorkspaceAction::Swap(maybe_id, dir) => {
                        wm.write().unwrap().swap_in_direction(
                            &rt,
                            &config,
                            &displays[0],
                            None,
                            dir,
                        );
                    }
                },
                Action::SaveSession => {
                    session::save_session(&wm.read().unwrap().workspaces);
                    info!("Saved session!");
                }
                Action::LoadSession => {
                    wm.write().unwrap().workspaces = session::load_session(tx.clone()).unwrap();
                    info!("Loaded session!");

                    let mut windows = Vec::new();

                    for ws in &wm.read().unwrap().workspaces {
                        for node in ws.graph.nodes.values() {
                            if let GraphNode::Window(win_id) = node {
                                windows.push(Window::new(*win_id));
                            }
                        }
                    }

                    for window in windows {
                        wm.write()
                            .unwrap()
                            .manage(&rt, &config, &displays[0], window);
                    }

                    wm.read()
                        .unwrap()
                        .render(&config, displays[0].get_size(&config));
                }
                Action::ShowTaskbars => {
                    for display in &displays {
                        display.show_taskbar();
                    }
                }
                Action::HideTaskbars => {
                    for display in &displays {
                        display.hide_taskbar();
                    }
                }
                Action::UpdateConfig { key, update_fn } => {
                    update_fn.0(&mut config);
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
                            String::from_lua(rt.eval("_G.__stdout_buf").unwrap(), rt.rt).unwrap();

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
            },
            Event::RenderGraph => {
                wm.read()
                    .unwrap()
                    .render(&config, displays[0].get_size(&config));
            }
            Event::Exit => {
                for display in &displays {
                    display.show_taskbar();
                }

                WindowEventLoop::stop();
                KeybindingEventLoop::stop();

                wm.write().unwrap().cleanup();

                break;
            }
        }
    }
}
