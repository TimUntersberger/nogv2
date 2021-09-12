use std::{
    mem::{self, MaybeUninit},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
};

use chrono::Duration;
use nog_protocol::BarContent;

use crate::{
    config::Config,
    event::Event,
    lua::{self, LuaRuntime},
    platform::WindowId,
    thread_safe::ThreadSafe,
    types::ThreadSafeWindowManagers,
    window_manager::WindowManager,
};

#[derive(Clone)]
pub struct State {
    pub tx: Sender<Event>,
    // pub rx: Receiver<Event>,
    pub wms: ThreadSafeWindowManagers,
    pub bar_content: ThreadSafe<BarContent>,
    // pub bar_content_timer: (timer::Guard, timer::Timer),
    // pub rt: LuaRuntime<'static>,
    pub config: ThreadSafe<Config>,
}

impl State {
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            tx,
            wms: ThreadSafeWindowManagers::default(),
            bar_content: Default::default(),
            config: Default::default(),
        }
    }

    pub fn win_is_managed(&self, win_id: WindowId) -> bool {
        self.wms
            .read()
            .iter()
            .any(|wm| wm.read().has_window(win_id))
    }

    /// Doesn't call the function if no wm has the window
    pub fn with_wm_containing_win_mut<T>(
        &self,
        win_id: WindowId,
        f: impl Fn(&mut WindowManager) -> T,
    ) -> Option<T> {
        self.wms
            .read()
            .iter()
            .find(|wm| wm.read().has_window(win_id))
            .map(|wm| f(&mut wm.write()))
    }

    pub fn with_focused_wm_mut<T>(&self, f: impl Fn(&mut WindowManager) -> T) -> T {
        f(&mut self.wms.read()[0].write())
    }
}
