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
    bar::Bar,
    config::Config,
    display::{Display, DisplayId},
    event::Event,
    lua::{self, LuaRuntime},
    platform::{MonitorId, NativeMonitor, WindowId},
    thread_safe::ThreadSafe,
    types::ThreadSafeWindowManagers,
    window_manager::WindowManager,
};

#[derive(Clone)]
pub struct State {
    pub tx: Sender<Event>,
    pub displays: ThreadSafe<Vec<Display>>,
    pub bar_content: ThreadSafe<BarContent>,
    pub config: ThreadSafe<Config>,
}

impl State {
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            tx,
            displays: Default::default(),
            bar_content: Default::default(),
            config: Default::default(),
        }
    }

    pub fn win_is_managed(&self, win_id: WindowId) -> bool {
        self.displays.read().iter().any(|d| d.wm.has_window(win_id))
    }

    /// Doesn't call the function if no wm has the window
    pub fn with_dsp_containing_win_mut<T>(
        &self,
        win_id: WindowId,
        f: impl Fn(&mut Display) -> T,
    ) -> Option<T> {
        self.displays
            .write()
            .iter_mut()
            .find(|d| d.wm.has_window(win_id))
            .map(f)
    }

    pub fn with_focused_dsp_mut<T>(&self, f: impl Fn(&mut Display) -> T) -> T {
        f(&mut self.displays.write()[0])
    }

    /// Doesn't call the function if no wm exists on the display
    pub fn with_dsp<T>(&self, id: DisplayId, f: impl Fn(&Display) -> T) -> Option<T> {
        self.displays.read().iter().find(|d| d.id == id).map(f)
    }
}
