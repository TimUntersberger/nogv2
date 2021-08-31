use std::{collections::HashMap, sync::mpsc::Sender};

use crate::{cleanup::WindowCleanup, config::Config, event::Event, platform::{NativeWindow, Window, WindowId}, workspace::{Workspace, WorkspaceId}};

pub struct WindowManager {
    tx: Sender<Event>,
    pub workspaces: Vec<Workspace>,
    pub focused_workspace_id: WorkspaceId,
    pub window_cleanup: HashMap<WindowId, WindowCleanup>,
}

impl WindowManager {
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            workspaces: vec![Workspace::new(tx.clone())],
            focused_workspace_id: WorkspaceId(0),
            window_cleanup: HashMap::new(),
            tx,
        }
    }

    pub fn get_focused_workspace_mut<'a>(&'a mut self) -> &'a mut Workspace {
        &mut self.workspaces[self.focused_workspace_id.0]
    }

    pub fn get_focused_workspace<'a>(&'a self) -> &'a Workspace {
        &self.workspaces[self.focused_workspace_id.0]
    }

    pub fn manage(&mut self, config: &Config, win: Window) {
        let size = win.get_size();
        let pos = win.get_position();
        let cleanup = self.window_cleanup.entry(win.get_id()).or_default();

        cleanup.reset_transform = Some(Box::new(move || {
            win.reposition(pos);
            win.resize(size);
        }));

        if config.remove_decorations {
            cleanup.add_decorations = Some(win.remove_decorations())
        }
    }

    pub fn unmanage(&mut self, win_id: WindowId) {
        if let Some(cleanup) = self.window_cleanup.get(&win_id) {
            if let Some(f) = cleanup.add_decorations.as_ref() {
                f();
            }

            if let Some(f) = cleanup.reset_transform.as_ref() {
                f();
            }
        }

        self.window_cleanup.remove(&win_id);
    }
}
