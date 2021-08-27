use event::Event;
use keybinding_event_loop::KeybindingEventLoop;
use log::info;
use mlua::FromLua;
use server::Server;
use std::{
    sync::mpsc::{channel, Sender},
    thread,
};
use window_event_loop::WindowEventLoop;
use lua::graph_proxy::GraphProxy;

use crate::{config::Config, platform::NativeWindow, window_event_loop::WindowEventKind};

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

mod config;
mod event;
mod key;
mod key_combination;
mod keybinding;
mod keybinding_event_loop;
mod logging;
mod lua;
mod modifiers;
mod platform;
mod server;
mod window_event_loop;

fn main() {
    logging::init().expect("Failed to initialize logging");
    info!("Initialized logging");

    let (tx, rx) = channel::<Event>();
    let mut rt = lua::init(tx.clone()).unwrap();
    let mut config = Config::default();

    // lua::repl::spawn(tx.clone());
    // info!("Repl started");

    Server::spawn(tx.clone());
    info!("IPC Server started");

    WindowEventLoop::spawn(tx.clone());
    info!("Window event loop spawned");

    KeybindingEventLoop::spawn(tx.clone());
    info!("Keybinding event loop spawned");

    info!("Starting main event loop");
    while let Ok(event) = rx.recv() {
        match event {
            Event::Window(win_event) => {
                //info!("{:?} {:?}", win_event.kind, win_event.window);
                match win_event.kind {
                    WindowEventKind::Created => {
                        let (width, height) = win_event.window.get_size();

                        if width >= config.min_width && height >= config.min_height {
                            info!("{}", win_event.window.get_title());
                        }

                        rt.call_fn("nog.layout", (GraphProxy, "created", win_event.window.0.0)).unwrap();
                    },
                    WindowEventKind::Deleted => {},
                };
            }
            Event::Keybinding(kb) => {
                info!("Keybinding {}", kb.to_string());
            }
            Event::Action(action) => match action {
                event::Action::UpdateConfig { key, update_fn } => {
                    update_fn.0(&mut config);
                    info!("Updated config property: {:#?}", key);
                }
                event::Action::ExecuteLua {
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
                            String::from_lua(rt.eval("_G.__stdout_buf").unwrap(), rt.rt)
                                .unwrap();

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
                event::Action::CreateKeybinding {
                    mode,
                    key_combination,
                } => {
                    KeybindingEventLoop::add_keybinding(key_combination.get_id());
                    info!("Created {:?} keybinding: {:#?}", mode, key_combination);
                }
                event::Action::RemoveKeybinding { key } => {
                    // KeybindingEventLoop::remove_keybinding(key_combination.get_id());
                    info!("Removed keybinding: {:#?}", key);
                }
            },
        }
    }
}
