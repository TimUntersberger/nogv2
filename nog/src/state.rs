use std::sync::{atomic::AtomicBool, mpsc::Sender, Arc};

use nog_protocol::BarContent;

use crate::{
    config::Config,
    display::{Display, DisplayId},
    event::Event,
    keybinding::Keybinding,
    lua::LuaEvent,
    notification::NotificationManager,
    platform::WindowId,
    thread_safe::ThreadSafe,
    workspace::{Workspace, WorkspaceId},
};

#[derive(Debug, Clone)]
/// You can clone the state without any worries.
pub struct State {
    pub awake: Arc<AtomicBool>,
    pub tx: Sender<Event>,
    pub displays: ThreadSafe<Vec<Display>>,
    pub bar_content: ThreadSafe<BarContent>,
    pub keybindings: ThreadSafe<Vec<Keybinding>>,
    pub config: ThreadSafe<Config>,
}

impl State {
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            awake: Arc::new(AtomicBool::new(true)),
            tx,
            displays: Default::default(),
            keybindings: Default::default(),
            bar_content: Default::default(),
            config: Default::default(),
        }
    }

    pub fn awake(&self) {
        self.awake.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn hibernate(&self) {
        self.awake.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_awake(&self) -> bool {
        self.awake.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn win_is_managed(&self, win_id: WindowId) -> bool {
        self.displays.read().iter().any(|d| d.wm.has_window(win_id))
    }

    /// Doesn't call the function if none was found
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

    /// Doesn't call the function if none was found
    pub fn with_dsp_containing_ws_mut<T>(
        &self,
        ws_id: WorkspaceId,
        f: impl Fn(&mut Display) -> T,
    ) -> Option<T> {
        self.displays
            .write()
            .iter_mut()
            .find(|d| d.wm.get_ws_by_id(ws_id).is_some())
            .map(f)
    }

    pub fn with_focused_dsp<T>(&self, f: impl Fn(&Display) -> T) -> T {
        f(&self.displays.read()[0])
    }

    pub fn with_focused_dsp_mut<T>(&self, f: impl Fn(&mut Display) -> T) -> T {
        f(&mut self.displays.write()[0])
    }

    pub fn get_focused_dsp_id(&self) -> DisplayId {
        self.displays.read()[0].id.clone()
    }

    pub fn get_focused_ws_id(&self) -> Option<WorkspaceId> {
        self.with_focused_dsp(|dsp| dsp.wm.focused_workspace_id)
    }

    /// Doesn't call the function if display doesn't exist
    pub fn with_dsp<T>(&self, id: DisplayId, f: impl Fn(&Display) -> T) -> Option<T> {
        self.displays.read().iter().find(|d| d.id == id).map(f)
    }

    /// Doesn't call the function if display doesn't exist
    pub fn with_dsp_mut<T>(&self, id: DisplayId, f: impl Fn(&mut Display) -> T) -> Option<T> {
        self.displays.write().iter_mut().find(|d| d.id == id).map(f)
    }

    /// Doesn't call the function if the workspace doesn't exist
    pub fn with_ws<T>(&self, id: WorkspaceId, f: impl Fn(&Workspace) -> T) -> Option<T> {
        self.displays
            .read()
            .iter()
            .map(|d| d.wm.get_ws_by_id(id))
            .flatten()
            .next()
            .map(f)
    }

    /// Doesn't call the function if the workspace doesn't exist
    pub fn with_ws_mut<T>(&self, id: WorkspaceId, f: impl Fn(&mut Workspace) -> T) -> Option<T> {
        self.displays
            .write()
            .iter_mut()
            .map(|d| d.wm.get_ws_by_id_mut(id))
            .flatten()
            .next()
            .map(f)
    }
}
