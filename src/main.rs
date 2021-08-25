use std::{sync::mpsc::{Sender, channel}, thread};
use event::Event;
use log::info;
use window_event_loop::WindowEventLoop;
use keybinding_event_loop::KeybindingEventLoop;

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

mod window_event_loop;
mod keybinding_event_loop;
mod platform;
mod logging;
mod event;
mod lua;

fn main() {
    logging::init().expect("Failed to initialize logging");
    info!("Initialized logging");

    let (tx, rx) = channel::<Event>();

    let rt = lua::init().unwrap();

    rt.eval("nog.say('hello', 2, 'what')").unwrap();

    WindowEventLoop::spawn(tx.clone());
    info!("Window event loop spawned");

    KeybindingEventLoop::spawn(tx.clone());
    info!("Keybinding event loop spawned");

    info!("Starting main event loop");
    while let Ok(event) = rx.recv() {
        match event {
            Event::Window(win_event) => {
                info!("{:?} {:?}", win_event.kind, win_event.window);
            },
            Event::Keybinding(kb) => {
                info!("Keybinding {}", kb.to_string());
            },
            Event::Action(_action) => {
                info!("Action");
            }
        }
    }
}
