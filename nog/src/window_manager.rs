use std::{collections::HashMap, mem};

use log::info;
use mlua::FromLua;

use crate::{
    cleanup::{WindowCleanup, WorkspaceCleanup},
    config::Config,
    direction::Direction,
    lua::{graph_proxy::GraphProxy, LuaRuntime},
    platform::{Area, NativeWindow, Window, WindowId},
    workspace::{Workspace, WorkspaceId},
};

#[derive(Debug, Clone)]
pub enum WindowManagerError {
    LayoutFunctionError(String),
}
pub type WindowManagerResult<T = ()> = Result<T, WindowManagerError>;

#[derive(Debug)]
pub struct WindowManager {
    pub workspaces: Vec<Workspace>,
    pub focused_workspace_id: WorkspaceId,
    pub window_cleanup: HashMap<WindowId, WindowCleanup>,
    pub workspace_cleanup: HashMap<WorkspaceId, WorkspaceCleanup>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            workspaces: vec![Workspace::new(WorkspaceId(1), "master_slave")],
            focused_workspace_id: WorkspaceId(1),
            window_cleanup: HashMap::new(),
            workspace_cleanup: HashMap::new(),
        }
    }

    pub fn get_focused_workspace_mut(&mut self) -> &mut Workspace {
        let id = self.focused_workspace_id;
        self.workspaces.iter_mut().find(|ws| ws.id == id).unwrap()
    }

    fn remove_workspace(&mut self, id: WorkspaceId) {
        for (idx, ws) in self.workspaces.iter().enumerate() {
            if ws.id == id {
                self.workspaces.remove(idx);
                break;
            }
        }
    }

    pub fn change_workspace(&mut self, id: WorkspaceId) {
        if self.focused_workspace_id == id {
            return;
        }

        match self.get_ws_by_id(id) {
            Some(ws) => ws.unminimize(),
            None => self.workspaces.push(Workspace::new(id, "master_slave")),
        };

        let old_ws_id = mem::replace(&mut self.focused_workspace_id, id);
        let old_ws = self.get_ws_by_id(old_ws_id).unwrap();

        if old_ws.is_empty() {
            self.remove_workspace(old_ws_id);
        } else {
            let ws = self.get_ws_by_id(old_ws_id).unwrap();
            ws.minimize();
        }
    }

    pub fn focus_window(&mut self, id: WindowId) -> bool {
        for ws in self.workspaces.iter_mut() {
            if ws.focus_window(id).is_ok() {
                let id = ws.id;
                self.change_workspace(id);
                return true;
            }
        }

        false
    }

    pub fn get_focused_workspace(&self) -> &Workspace {
        let id = self.focused_workspace_id;
        self.workspaces.iter().find(|ws| ws.id == id).unwrap()
    }

    pub fn get_ws_by_id(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.iter().find(|ws| ws.id == id)
    }

    pub fn get_ws_by_id_mut(&mut self, id: WorkspaceId) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|ws| ws.id == id)
    }

    pub fn has_window(&self, id: WindowId) -> bool {
        self.workspaces
            .iter()
            .map(|ws| ws.has_window(id))
            .any(|x| x)
    }

    pub fn manage(
        &mut self,
        rt: &LuaRuntime,
        config: &Config,
        ws_id: Option<WorkspaceId>,
        area: Area,
        win: Window,
    ) -> WindowManagerResult {
        let cleanup = self.window_cleanup.entry(win.get_id()).or_default();

        if win.is_maximized() {
            win.restore_placement();
            let size = win.get_size();
            let pos = win.get_position();
            cleanup.reset_transform = Some(Box::new(move || {
                win.reposition(pos);
                win.resize(size);
                win.maximize();
            }));
        } else {
            let size = win.get_size();
            let pos = win.get_position();
            cleanup.reset_transform = Some(Box::new(move || {
                win.reposition(pos);
                win.resize(size);
            }));
        }

        if config.remove_decorations {
            cleanup.add_decorations = Some(win.remove_decorations());
        }

        self.organize(
            rt,
            config,
            ws_id,
            area,
            String::from("managed"),
            win.get_id(),
        )
    }

    pub fn swap_in_direction(
        &mut self,
        rt: &LuaRuntime,
        config: &Config,
        area: Area,
        maybe_id: Option<WindowId>,
        dir: Direction,
    ) -> WindowManagerResult {
        let id = maybe_id.or_else(|| {
            self.get_focused_workspace()
                .get_focused_node()
                .and_then(|node| node.try_get_window_id())
        });

        if let Some(id) = id {
            self.organize(rt, config, None, area, String::from("swapped"), (id, dir))?;
        }

        Ok(())
    }

    /// Only renders the visible workspace
    pub fn render(&self, config: &Config, area: Area) {
        self.get_focused_workspace().render(config, area);
    }

    pub fn organize<TArgs: mlua::ToLuaMulti<'static>>(
        &mut self,
        rt: &LuaRuntime,
        config: &Config,
        ws_id: Option<WorkspaceId>,
        area: Area,
        reason: String,
        args: TArgs,
    ) -> WindowManagerResult {
        let ws_id = ws_id.unwrap_or_else(|| self.focused_workspace_id.clone());
        let mut workspace = self.get_ws_by_id_mut(ws_id).unwrap();
        // We need to use the scope here to make the rust type system happy.
        // scope drops the userdata when the function has finished.
        rt.lua
            .scope(|scope| {
                let ud = scope.create_nonstatic_userdata(GraphProxy(&mut workspace.graph))?;
                mlua::Function::from_lua(
                    rt.lua
                        .load(&format!(
                            "nog.__organize({}, '{}')",
                            workspace.id.0, &workspace.layout_name
                        ))
                        .eval()?,
                    rt.lua,
                )?
                .call((ud, reason, args))
            })
            .map_err(|e| WindowManagerError::LayoutFunctionError(e.to_string()))?;

        if workspace.graph.dirty {
            info!("Have to rerender!");
            workspace.render(config, area);
            println!("{}", &workspace.graph);
            workspace.graph.dirty = false;
        }

        Ok(())
    }

    pub fn unmanage(
        &mut self,
        rt: &LuaRuntime,
        config: &Config,
        area: Area,
        win_id: WindowId,
    ) -> WindowManagerResult {
        if let Some(cleanup) = self.window_cleanup.get(&win_id) {
            info!("Doing cleanup for {}", win_id);
            if let Some(f) = cleanup.add_decorations.as_ref() {
                f();
            }

            if let Some(f) = cleanup.reset_transform.as_ref() {
                f();
            }
        }

        self.organize(rt, config, None, area, String::from("unmanaged"), win_id)?;

        self.window_cleanup.remove(&win_id);

        Ok(())
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

        self.focused_workspace_id = WorkspaceId(1);
        self.workspaces = vec![Workspace::new(WorkspaceId(1), "master_slave")];
    }
}
