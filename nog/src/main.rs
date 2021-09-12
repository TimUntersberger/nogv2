use action::{Action, WindowAction, WorkspaceAction};
use chrono::Duration;
use event::Event;
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
    state::State,
    thread_safe::ThreadSafe,
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

mod action;
mod cleanup;
mod config;
mod direction;
mod display;
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
mod platform;
mod rgb;
mod server;
mod session;
mod state;
mod thread_safe;
mod types;
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
            let fg = tbl
                .get::<&str, i32>("fg")
                .map(RGB::from_hex)
                .unwrap_or(default_fg);
            let bg = tbl
                .get::<&str, i32>("bg")
                .map(RGB::from_hex)
                .unwrap_or(default_bg);

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

    let (tx, rx) = channel();
    // The timer stops repeating once the guard and the timer are dropped, so we have to hold both
    // of them until program termination.
    let _bar_content_timer = {
        let timer = timer::Timer::new();
        let tx = tx.clone();
        (
            timer.schedule_repeating(Duration::milliseconds(100), move || {
                tx.send(Event::RenderBarLayout).unwrap();
            }),
            timer,
        )
    };

    let state = State::new(tx.clone());

    state.wms.write().push(ThreadSafe::new(WindowManager::new(
        tx.clone(),
        Display::new(true),
    )));

    let rt = lua::init(state.clone()).unwrap();

    // Only really used in development to make sure everything is cleaned up
    {
        let tx = tx.clone();
        ctrlc::set_handler(move || tx.send(Event::Exit).unwrap());
    }

    // Run the config
    if let Err(e) = rt.eval("dofile(nog.config_path .. '/lua/config.lua')") {
        error!("config error: {}", e);
    }

    if state.config.read().remove_task_bar {
        tx.send(Event::Action(Action::HideTaskbars)).unwrap();
    }

    // lua::repl::spawn(tx.clone());
    // info!("Repl started");

    Server::spawn(tx.clone(), state.bar_content.clone());
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
                    let win_id = win_event.window.get_id();
                    state.with_wm_containing_win_mut(win_id, |wm| {
                        let workspace = wm.get_focused_workspace_mut();
                        if workspace.focus_window(win_id).is_ok() {
                            info!("Focused window with id {}", win_event.window.get_id());
                            win_event.window.focus();
                        }
                    });
                }
                WindowEventKind::Created => {
                    let win = win_event.window;
                    let size = win.get_size();

                    if size.width >= state.config.read().min_width
                        && size.height >= state.config.read().min_height
                    {
                        info!("'{}' created", win.get_title());
                        state.with_focused_wm_mut(|wm| {
                            wm.manage(&rt, &state.config.read(), win);
                        });
                    }
                }
                WindowEventKind::Deleted => {
                    let win_id = win_event.window.get_id();
                    state.with_wm_containing_win_mut(win_id, |wm| {
                        wm.organize(
                            &rt,
                            &state.config.read(),
                            None,
                            String::from("deleted"),
                            win_id,
                        );
                    });
                }
                WindowEventKind::Minimized => {
                    let win_id = win_event.window.get_id();

                    state.with_wm_containing_win_mut(win_id, |wm| {
                        wm.unmanage(&rt, &state.config.read(), win_id);
                        info!("'{}' minimized", win_event.window.get_title());
                    });
                }
            },
            Event::RenderBarLayout => {
                let default_fg = state.config.read().get_text_color();
                let default_bg = state.config.read().color;

                macro_rules! convert_sections {
                    {$(($ident:ident, $s:expr)),*} => {
                        {
                            let mut result = Vec::new();
                            $(
                                //TODO: proper error handling instead of expecting values
                                for value in rt
                                    .lua
                                    .named_registry_value::<str, mlua::Table>($s)
                                    .expect(&format!("Registry value of {} bar layout section missing", $s))
                                    .sequence_values()
                                    .map(|v| mlua::Function::from_lua(v.unwrap(), &rt.lua)
                                            .expect("Has to be a function")
                                            .call::<(), mlua::Value>(())
                                            .expect("Cannot error")

                                    ) {
                                        match value {
                                            tbl @ mlua::Value::Table(..) => {
                                                let tbl = mlua::Table::from_lua(tbl, rt.lua).unwrap();
                                                for value in tbl.sequence_values() {
                                                    result.push(
                                                        lua_value_to_bar_item(
                                                            &rt.lua,
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
                                                        &rt.lua,
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

                *state.bar_content.write() = BarContent {
                    bg: state.config.read().color.0,
                    items,
                };
            }
            Event::Keybinding(kb) => {
                info!("Received keybinding {}", kb.to_string());

                let cb = rt
                    .lua
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
            Event::Action(action) => action.handle(&state, &rt),
            Event::RenderGraph => {
                for wm in state.wms.read().iter() {
                    wm.read().render(&state.config.read());
                }
            }
            Event::Exit => {
                let wms = state.wms.read();

                for wm in wms.iter() {
                    let mut wm = wm.write();
                    wm.display.show_taskbar();
                    wm.cleanup();
                }

                WindowEventLoop::stop();
                KeybindingEventLoop::stop();

                break;
            }
        }
    }
}
