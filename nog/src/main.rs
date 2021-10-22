#![feature(windows_process_extensions_raw_arg)]

use action::Action;
use chrono::Duration;
use event::Event;
use keybinding_event_loop::KeybindingEventLoop;
use log::{error, info};
use mlua::FromLua;
use nog_protocol::{BarContent, BarItem, BarItemAlignment};
use rgb::Rgb;
use server::Server;
use std::{
    process::Command,
    sync::mpsc::{channel, Sender},
    thread,
};
use window_event_loop::WindowEventLoop;

use crate::{
    lua::lua_error_to_string,
    paths::get_bin_path,
    platform::{Api, NativeApi, NativeWindow},
    state::State,
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
mod bar;
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
mod server;
mod session;
mod state;
mod thread_safe;
mod window_event_loop;
mod window_manager;
mod workspace;

fn lua_value_to_bar_item(
    lua: &mlua::Lua,
    align: BarItemAlignment,
    value: mlua::Value<'_>,
    default_fg: Rgb,
    default_bg: Rgb,
) -> mlua::Result<BarItem> {
    Ok(match value {
        tbl @ mlua::Value::Table(..) => {
            let tbl = mlua::Table::from_lua(tbl, lua).unwrap();
            let text = String::from_lua(tbl.get(1).unwrap_or(mlua::Value::Nil), lua).unwrap();
            let fg = tbl
                .get::<&str, i32>("fg")
                .map(Rgb::from_hex)
                .unwrap_or(default_fg);

            let bg = tbl
                .get::<&str, i32>("bg")
                .map(Rgb::from_hex)
                .unwrap_or(default_bg);

            BarItem {
                text,
                alignment: align,
                fg: fg.0,
                bg: bg.0,
            }
        }
        value => {
            let text = lua
                .coerce_string(value)?
                .map(|s| s.to_str().unwrap().to_string())
                .unwrap_or_else(|| String::from("nil"));

            BarItem {
                text,
                alignment: align,
                fg: default_fg.0,
                bg: default_bg.0,
            }
        }
    })
}

#[derive(Debug)]
enum Error {
    Ctrlc(ctrlc::Error),
    Lua(mlua::Error),
}

fn main() {
    dbg!(failable_main());
}

fn failable_main() -> Result<(), Error> {
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

    info!("Looking for displays to use");
    *state.displays.write() = Api::get_displays();

    if state.displays.read().is_empty() {
        panic!("Couldn't find any displays the fuck?");
    }

    let rt = lua::init(state.clone()).map_err(Error::Lua)?;

    // Only really used in development to make sure everything is cleaned up
    {
        let tx = tx.clone();
        ctrlc::set_handler(move || tx.send(Event::Exit).unwrap()).map_err(Error::Ctrlc)?;
    }

    // Run the config
    if let Err(e) = rt.eval("dofile(nog.config_path .. '/config/init.lua')") {
        log::error!("Failed to execute config: {}", lua_error_to_string(e));
    }

    tx.send(Event::ConfigFinished).unwrap();

    Server::spawn(tx.clone(), state.clone());
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
                    if state.is_awake() {
                        let win_id = win_event.window.get_id();
                        state.with_dsp_containing_win_mut(win_id, |d| {
                            if d.wm.focus_window(win_id) {
                                info!("Focused window with id {}", win_event.window.get_id());
                                win_event.window.focus();
                            }
                        });
                    }
                }
                WindowEventKind::Created => {
                    if state.is_awake() {
                        let win = win_event.window;

                        if state.with_dsp_containing_win_mut(win.get_id(), |_| ()).is_some() {
                            log::debug!("Window is already managed");
                            continue;
                        }

                        let title = win.get_title();

                        if title == "nog_bar" {
                            continue;
                        }

                        if title == "nog_menu" {
                            win.focus();
                            continue;
                        }

                        let size = win.get_size();

                        if size.width >= state.config.read().min_width
                            && size.height >= state.config.read().min_height
                        {
                            info!("'{}' created", win.get_title());
                            state.with_focused_dsp_mut(|d| {
                                let area = d.get_render_area(&state.config.read());
                                d.wm.manage(&rt, &state.config.read(), None, area, win).unwrap();
                            });
                        }
                    }
                }
                WindowEventKind::Deleted => {
                    if state.is_awake() {
                        let win_id = win_event.window.get_id();
                        state.with_dsp_containing_win_mut(win_id, |d| {
                            let area = d.get_render_area(&state.config.read());
                            d.wm.unmanage(&rt, &state.config.read(), area, win_id)
                                .unwrap();
                            info!("'{}' deleted", win_event.window.get_title());
                        });
                    }
                }
                WindowEventKind::Minimized => {
                    //TODO: Changing workspaces minimizes the windows of the previous workspace.
                    //This then causes this event for each window, resulting in unmanaging each
                    //window. Somehow ignore this event when changing workspaces.
                    //
                    // let win_id = win_event.window.get_id();

                    // state.with_dsp_containing_win_mut(win_id, |d| {
                    //     info!("Managed window {} got minimzed", win_id);
                    //     let area = d.get_render_area(&state.config.read());
                    //     d.wm.unmanage(&rt, &state.config.read(), area, win_id)
                    //         .unwrap();
                    //     info!("'{}' minimized", win_event.window.get_title());
                    // });
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
                                    .enumerate()
                                    .map(|(i, v)| mlua::Function::from_lua(v.unwrap(), &rt.lua)
                                            .expect("Has to be a function")
                                            .call::<(), mlua::Value>(())
                                            .unwrap_or_else(|e| {
                                                use mlua::ToLua;

                                                error!("Bar component {} in the {} section errored.\n\terror: {}", i, $s, match e {
                                                    mlua::Error::CallbackError { cause, .. } => cause.to_string(),
                                                    e => e.to_string(),
                                                });

                                                rt.lua.create_string("").unwrap().to_lua(rt.lua).unwrap()
                                            })

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
                    font_name: state.config.read().font_name.clone(),
                    font_size: state.config.read().font_size,
                    height: state.config.read().bar_height as usize,
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
                for d in state.displays.read().iter() {
                    let area = d.get_render_area(&state.config.read());
                    d.wm.render(&state.config.read(), area);
                }
            }
            Event::Exit => {
                for d in state.displays.write().iter_mut() {
                    d.show_taskbar();
                    d.wm.cleanup();
                }

                WindowEventLoop::stop();
                KeybindingEventLoop::stop();

                break;
            }
            Event::ConfigFinished => {
                if state.config.read().remove_task_bar {
                    tx.send(Event::Action(Action::HideTaskbars)).unwrap();
                }

                if state.config.read().display_app_bar {
                    tx.send(Event::Action(Action::ShowBars)).unwrap();
                }
            }
            Event::ShowMenu => {
                let mut path = get_bin_path();
                path.push("nog-menu.exe");

                Command::new(path)
                    .args(["-b", &format!("0x{:x}", state.config.read().color.to_hex())])
                    .args([
                        "-t",
                        if state.config.read().light_theme {
                            "0x000000"
                        } else {
                            "0xFFFFFF"
                        },
                    ])
                    .spawn()
                    .unwrap();
            }
        }
    }

    Ok(())
}
