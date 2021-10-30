use mlua::Lua;
use notify::Watcher;
use std::{
    path::PathBuf,
    sync::{
        mpsc::{channel, sync_channel, Sender, SyncSender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::event::{DeferedFunction, Event};

pub struct FileWatcher {
    tx: SyncSender<Event>,
    reg_key: Arc<mlua::RegistryKey>,
    cancel_tx: Option<SyncSender<()>>,
    path: PathBuf,
}

impl FileWatcher {
    pub fn new(path: PathBuf, tx: SyncSender<Event>, key: mlua::RegistryKey) -> Self {
        Self {
            reg_key: Arc::new(key),
            tx,
            cancel_tx: None,
            path,
        }
    }

    pub fn start(&mut self) {
        let (cancel_tx, cancel_rx) = sync_channel(1);

        self.cancel_tx = Some(cancel_tx);

        let path = self.path.clone();
        let reg_key = self.reg_key.clone();
        let event_tx = self.tx.clone();

        thread::spawn(move || {
            let (tx, rx) = channel();

            let mut watcher = notify::watcher(tx, Duration::from_millis(100)).unwrap();

            watcher
                .watch(&path, notify::RecursiveMode::Recursive)
                .unwrap();

            loop {
                if cancel_rx.try_recv().is_ok() {
                    break;
                }
                if let Ok(ev) = rx.try_recv() {
                    use notify::DebouncedEvent::*;

                    let (ev_name, args) = match ev {
                        Write(filename) => ("write", vec![filename]),
                        Create(filename) => ("create", vec![filename]),
                        Remove(filename) => ("remove", vec![filename]),
                        Rename(from, to) => ("rename", vec![from, to]),
                        _ => continue,
                    };

                    let reg_key = reg_key.clone();

                    event_tx
                        .send(Event::Defered(DeferedFunction::new(move |rt, _state| {
                            let cb: mlua::Function = rt.lua.registry_value(&reg_key).unwrap();

                            cb.call::<(&str, Vec<String>), ()>((
                                ev_name,
                                args.iter()
                                    .map(|path| path.to_string_lossy().to_string())
                                    .collect::<Vec<String>>(),
                            ))
                            .unwrap();
                        })))
                        .unwrap();
                }
            }
        });
    }

    pub fn stop(&mut self) {
        if let Some(tx) = &self.cancel_tx {
            tx.send(()).unwrap();
        }

        self.cancel_tx = None;
    }
}

impl mlua::UserData for FileWatcher {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("start", |_lua, this, _args: ()| {
            this.start();
            Ok(())
        });
        methods.add_method_mut("stop", |_lua, this, _args: ()| {
            this.stop();
            Ok(())
        });
    }
}
