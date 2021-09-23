use std::sync::mpsc::Sender;

use nog_protocol::BarContent;

use crate::{
    config::Config,
    display::{Display, DisplayId},
    event::Event,
    platform::WindowId,
    thread_safe::ThreadSafe,
    workspace::{Workspace, WorkspaceId},
};

#[derive(Clone)]
/// You can clone the state without any worries.
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

    pub fn with_focused_dsp_mut<T>(&self, f: impl Fn(&mut Display) -> T) -> T {
        f(&mut self.displays.write()[0])
    }

    pub fn get_focused_dsp_id(&self) -> DisplayId {
        self.displays.read()[0].id.clone()
    }

    /// Doesn't call the function if display doesn't exist
    pub fn with_dsp<T>(&self, id: DisplayId, f: impl Fn(&Display) -> T) -> Option<T> {
        self.displays.read().iter().find(|d| d.id == id).map(f)
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
}
