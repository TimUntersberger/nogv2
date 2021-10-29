use crate::config::Config;
use crate::direction::Direction;
use crate::graph::{Graph, GraphNode, GraphNodeGroupKind, GraphNodeId};
use crate::platform::{Area, NativeWindow, Window, WindowId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorkspaceState {
    Fullscreen,
    Normal,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkspaceId(pub usize);

#[derive(Debug)]
pub struct Workspace {
    pub id: WorkspaceId,
    /// Initially this is set to the id of the workspace
    pub display_name: String,
    pub layout_name: String,
    pub state: WorkspaceState,
    pub graph: Graph,
}

pub enum WorkspaceError {
    WindowNodeNotFound,
}

pub type WorkspaceResult<T = ()> = Result<T, WorkspaceError>;

impl Workspace {
    pub fn new(id: WorkspaceId, layout_name: &str) -> Self {
        Self {
            id,
            display_name: id.0.to_string(),
            layout_name: layout_name.to_string(),
            state: WorkspaceState::Normal,
            graph: Graph::new(),
        }
    }

    pub fn is_fullscreen(&self) -> bool {
        self.state == WorkspaceState::Fullscreen
    }

    pub fn get_focused_win(&self) -> Option<Window> {
        self.graph
            .get_focused_window_child(0)
            .and_then(|id| self.graph.get_node(id))
            .and_then(|n| n.try_get_window_id())
            .map(Window::new)
    }

    pub fn is_empty(&self) -> bool {
        // If the graph doesn't have any edges then only the root node can exist.
        self.graph.edges.is_empty()
    }

    pub fn windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.graph
            .nodes
            .values()
            .map(|n| n.try_get_window_id())
            .flatten()
    }

    pub fn hide(&self) {
        for win in self.windows() {
            Window::new(win).hide();
        }
    }

    pub fn minimize(&self) {
        for win in self.windows() {
            Window::new(win).minimize();
        }
    }

    pub fn unminimize(&self) {
        for win in self.windows() {
            Window::new(win).unminimize();
        }
    }

    pub fn show(&self) {
        for win in self.windows() {
            Window::new(win).show();
        }
    }

    pub fn get_focused_node(&self) -> Option<&GraphNode> {
        self.graph
            .get_focused_window_child(0)
            .and_then(|id| self.graph.get_node(id))
    }

    pub fn has_window(&self, id: WindowId) -> bool {
        self.graph.get_window_node(id).is_some()
    }

    pub fn render(&self, config: &Config, mut area: Area) {
        area.pos.x += config.outer_gap as isize;
        area.pos.y += config.outer_gap as isize;

        area.size.width -= config.outer_gap as usize * 2;
        area.size.height -= config.outer_gap as usize * 2;

        match &self.state {
            WorkspaceState::Fullscreen => {
                if let Some(win_node_id) = self.graph.get_focused_window_child(0) {
                    render_node(win_node_id, &self.graph, config, area);
                }
            }
            WorkspaceState::Normal => {
                render_node(self.graph.root_node_id, &self.graph, config, area);
            }
        }
    }

    fn focus_node(&mut self, id: GraphNodeId) {
        if let Some(parent) = self.graph.get_parent_node(id) {
            let idx = self
                .graph
                .get_children(parent)
                .iter()
                .enumerate()
                .find(|(_, c)| **c == id)
                .map(|(idx, _)| idx)
                .unwrap();

            match self.graph.get_node_mut(parent).unwrap() {
                GraphNode::Group { focus, .. } => {
                    *focus = idx;
                }
                _ => unreachable!(),
            };

            // If we focus a node in the graph we also need to set the focus of all the
            // parent nodes until we hit the root, because we can't be sure that the parent has focus.
            self.focus_node(parent);
        }
    }

    pub fn focus_window(&mut self, id: WindowId) -> WorkspaceResult {
        let node_id = self
            .graph
            .get_window_node(id)
            .ok_or(WorkspaceError::WindowNodeNotFound)?;

        self.focus_node(node_id);

        Ok(())
    }

    pub fn focus_in_direction(&mut self, dir: Direction) -> Option<GraphNodeId> {
        self.graph
            .get_focused_window_child(0)
            .and_then(|id| self.graph.get_window_node_in_direction(id, dir))
            .map(|node_id| {
                self.focus_node(node_id);
                node_id
            })
    }

    pub fn swap_in_direction(&mut self, dir: Direction) -> Option<GraphNodeId> {
        self.graph
            .get_focused_window_child(0)
            .and_then(|id| self.graph.get_window_node_in_direction(id, dir))
            .map(|node_id| {
                self.focus_node(node_id);
                node_id
            })
    }
}

fn render_node(id: GraphNodeId, graph: &Graph, config: &Config, mut area: Area) {
    let node = graph
        .get_node(id)
        .expect("Cannot render a node that doesn't exist");

    match node {
        GraphNode::Group { kind, .. } => {
            let children = graph.get_children(id);

            if children.is_empty() {
                return;
            }

            match kind {
                GraphNodeGroupKind::Row => {
                    let col_width = area.size.width / children.len();
                    let mut x = area.pos.x;
                    for child_id in children {
                        area.pos.x = x;
                        area.size.width = col_width;
                        render_node(child_id, graph, config, area);
                        x += col_width as isize;
                    }
                }
                GraphNodeGroupKind::Col => {
                    let row_height = area.size.height / children.len();
                    let mut y = area.pos.y;
                    for child_id in children {
                        area.pos.y = y;
                        area.size.height = row_height;
                        render_node(child_id, graph, config, area);
                        y += row_height as isize;
                    }
                }
            }
        }
        GraphNode::Window(win_id) => {
            area.pos.x += config.inner_gap as isize;
            area.pos.y += config.inner_gap as isize;
            area.size.width -= config.inner_gap as usize * 2;
            area.size.height -= config.inner_gap as usize * 2;

            log::trace!(
                "Rendering Window({}) x={} y={} width={} height={}",
                win_id,
                area.pos.x,
                area.pos.y,
                area.size.width,
                area.size.height
            );

            let win = Window::new(*win_id);
            win.reposition(area.pos);
            win.resize(area.size);
        }
    }
}
