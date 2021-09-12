use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, RwLock,
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

#[derive(Debug)]
pub enum StateError {
    LuaInitFailed(mlua::Error),
}

pub type StateResult<T = ()> = Result<T, StateError>;

pub struct State {
    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
    pub wms: ThreadSafeWindowManagers,
    pub bar_content: ThreadSafe<BarContent>,
    pub bar_content_timer: (timer::Guard, timer::Timer),
    pub rt: LuaRuntime<'static>,
    pub config: Config,
}

impl State {
    pub fn new() -> StateResult<Self> {
        let (tx, rx) = channel();
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
        let wms = ThreadSafeWindowManagers::default();
        let rt = lua::init(tx.clone(), wms.clone()).map_err(StateError::LuaInitFailed)?;

        Ok(Self {
            tx,
            rx,
            wms,
            bar_content: Default::default(),
            bar_content_timer,
            rt,
            config: Config::default(),
        })
    }

    /// Doesn't call the function if no wm has the window
    pub fn with_wm_containing_win_mut<T>(
        &self,
        win_id: WindowId,
        f: impl Fn(&mut WindowManager) -> T,
    ) -> Option<T> {
        self.wms
            .read()
            .unwrap()
            .iter()
            .find(|wm| wm.read().unwrap().has_window(win_id))
            .map(|wm| f(&mut wm.write().unwrap()))
    }

    pub fn with_focused_wm_mut<T>(&self, f: impl Fn(&mut WindowManager) -> T) -> T {
        f(&mut self.wms.read().unwrap()[0].write().unwrap())
    }
}