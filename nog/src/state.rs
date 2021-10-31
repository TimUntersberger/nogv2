use crate::{
    config::Config,
    display::{Display, DisplayId},
    event::Event,
    keybinding::Keybinding,
    platform::WindowId,
    thread_safe::ThreadSafe,
    workspace::{Workspace, WorkspaceId},
};
use nog_protocol::BarContent;
use std::sync::mpsc::SyncSender;

#[derive(Debug, Clone, PartialEq)]
pub enum StateMode {
    Awake,
    Hibernating,
    Initializing
}

/// You can clone the state without any worries.
#[derive(Debug, Clone)]
pub struct State {
    pub mode: ThreadSafe<StateMode>,
    pub tx: SyncSender<Event>,
    pub displays: ThreadSafe<Vec<Display>>,
    pub bar_content: ThreadSafe<BarContent>,
    pub keybindings: ThreadSafe<Vec<Keybinding>>,
    pub config: ThreadSafe<Config>,
}

impl State {
    pub fn new(tx: SyncSender<Event>) -> Self {
        Self {
            mode: ThreadSafe::new(StateMode::Initializing),
            tx,
            displays: Default::default(),
            keybindings: Default::default(),
            bar_content: Default::default(),
            config: Default::default(),
        }
    }

    pub fn awake(&self) {
        *self.mode.write() = StateMode::Awake;
    }

    pub fn hibernate(&self) {
        *self.mode.write() = StateMode::Hibernating;
    }

    pub fn is_awake(&self) -> bool {
        *self.mode.read() == StateMode::Awake
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
