use event::Event;
use keybinding_event_loop::KeybindingEventLoop;
use log::info;
use std::{
    sync::mpsc::{channel, Sender},
    thread,
};
use window_event_loop::WindowEventLoop;

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

mod event;
mod config;
mod keybinding_event_loop;
mod logging;
mod lua;
mod keybinding;
mod platform;
mod window_event_loop;

fn main() {
    logging::init().expect("Failed to initialize logging");
    info!("Initialized logging");

    let (tx, rx) = channel::<Event>();

    std::thread::spawn(move || {
        let mut rt = lua::init(tx.clone()).unwrap();
        lua::repl::start(&mut rt);
    });

    // WindowEventLoop::spawn(tx.clone());
    // info!("Window event loop spawned");

    // KeybindingEventLoop::spawn(tx.clone());
    // info!("Keybinding event loop spawned");

    info!("Starting main event loop");
    while let Ok(event) = rx.recv() {
        match event {
            Event::Window(win_event) => {
                info!("{:?} {:?}", win_event.kind, win_event.window);
            }
            Event::Keybinding(kb) => {
                info!("Keybinding {}", kb.to_string());
            }
            Event::Action(action) => {
                match action {
                    event::Action::UpdateConfig { 
                        key, 
                        update_fn 
                    } => {
                        info!("Updated config property: {:#?}", key);
                    },
                    event::Action::CreateKeybinding {
                        mode,
                        key
                    } => {
                        info!("Created {:?} keybinding: {:#?}", mode, key);
                    },
                    event::Action::RemoveKeybinding {
                        key
                    } => {
                        info!("Removed keybinding: {:#?}", key);
                    }
                }
            }
        }
    }
}
