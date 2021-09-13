use std::{collections::HashMap, mem, sync::mpsc::Sender};

use log::info;
use mlua::FromLua;

use crate::{
    cleanup::{DisplayCleanup, WindowCleanup, WorkspaceCleanup},
    config::Config,
    direction::Direction,
    event::Event,
    graph::{Graph, GraphNode, GraphNodeGroupKind, GraphNodeId},
    lua::{graph_proxy::GraphProxy, LuaRuntime},
    platform::{Area, Monitor, NativeMonitor, NativeWindow, Position, Size, Window, WindowId},
    workspace::{Workspace, WorkspaceId},
};

pub enum WindowManagerError {
    LayoutFunctionError(String),
}
pub type WindowManagerResult<T = ()> = Result<T, WindowManagerError>;

pub struct WindowManager {
    tx: Sender<Event>,
    pub workspaces: Vec<Workspace>,
    pub focused_workspace_id: WorkspaceId,
    pub window_cleanup: HashMap<WindowId, WindowCleanup>,
    pub workspace_cleanup: HashMap<WorkspaceId, WorkspaceCleanup>,
}

impl WindowManager {
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            workspaces: vec![Workspace::new(WorkspaceId(1), tx.clone())],
            focused_workspace_id: WorkspaceId(0),
            window_cleanup: HashMap::new(),
            workspace_cleanup: HashMap::new(),
            tx,
        }
    }

    pub fn get_focused_workspace_mut<'a>(&'a mut self) -> &'a mut Workspace {
        &mut self.workspaces[self.focused_workspace_id.0]
    }

    pub fn get_focused_workspace<'a>(&'a self) -> &'a Workspace {
        &self.workspaces[self.focused_workspace_id.0]
    }

    pub fn has_window(&self, id: WindowId) -> bool {
        self.workspaces
            .iter()
            .map(|ws| ws.has_window(id))
            .any(|x| x)
    }

    pub fn manage(&mut self, rt: &LuaRuntime, config: &Config, area: Area, win: Window) {
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

        self.organize(&rt, config, None, area, String::from("managed"), win.get_id());
    }

    pub fn swap_in_direction(
        &mut self,
        rt: &LuaRuntime,
        config: &Config,
        area: Area,
        maybe_id: Option<WindowId>,
        dir: Direction,
    ) {
        let id = maybe_id.or_else(|| {
            self.get_focused_workspace()
                .get_focused_node()
                .and_then(|node| node.try_get_window_id())
        });

        if let Some(id) = id {
            self.organize(rt, config, None, area, String::from("swapped"), (id, dir));
        }
    }

    /// Only renders the visible workspace
    pub fn render(&self, config: &Config, area: Area) {
        self.get_focused_workspace().render(config, area);
    }

    pub fn organize<TArgs: mlua::ToLuaMulti<'static>>(
        &mut self,
        rt: &LuaRuntime,
        config: &Config,
        maybe_workspace: Option<&mut Workspace>,
        area: Area,
        reason: String,
        args: TArgs,
    ) -> WindowManagerResult {
        let workspace = maybe_workspace.unwrap_or_else(|| self.get_focused_workspace_mut());
        // We need to use the scope here to make the rust type system happy.
        // scope drops the userdata when the function has finished.
        rt.lua
            .scope(|scope| {
                let ud = scope.create_nonstatic_userdata(GraphProxy(&mut workspace.graph))?;
                mlua::Function::from_lua(rt.lua.load("nog.layout").eval()?, rt.lua)?
                    .call((ud, reason, args))
            })
            .map_err(|e| WindowManagerError::LayoutFunctionError(e.to_string()))?;

        if workspace.graph.dirty {
            info!("Have to rerender!");
            workspace.render(&config, area);
            println!("{}", &workspace.graph);
            workspace.graph.dirty = false;
        }

        Ok(())
    }

    pub fn unmanage(&mut self, rt: &LuaRuntime, config: &Config, area: Area, win_id: WindowId) {
        if let Some(cleanup) = self.window_cleanup.get(&win_id) {
            if let Some(f) = cleanup.add_decorations.as_ref() {
                f();
            }

            if let Some(f) = cleanup.reset_transform.as_ref() {
                f();
            }
        }

        self.organize(&rt, config, None, area, String::from("unmanaged"), win_id);

        self.window_cleanup.remove(&win_id);
    }

    pub fn cleanup(&mut self) {
        for (_, v) in mem::take(&mut self.window_cleanup) {
            if let Some(f) = v.add_decorations {
                f();
            }
            if let Some(f) = v.reset_transform {
                f();
            }
        }
    }
}
