use crate::config::Config;
use crate::direction::Direction;
use crate::event::{Action, Event, WindowAction};
use crate::graph::{Graph, GraphNode, GraphNodeGroupKind, GraphNodeId};
use crate::platform::{NativeWindow, Window, WindowId, WindowPosition, WindowSize};
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct WorkspaceId(pub usize);

pub struct Workspace {
    pub graph: Graph,
    pub focused_node_id: Option<GraphNodeId>,
    tx: Sender<Event>,
}

pub enum WorkspaceError {
    WindowNodeNotFound,
}

pub type WorkspaceResult<T = ()> = Result<T, WorkspaceError>;

impl Workspace {
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            graph: Graph::new(),
            focused_node_id: None,
            tx,
        }
    }

    pub fn get_focused_node(&self) -> Option<&GraphNode> {
        self.focused_node_id.and_then(|id| self.graph.get_node(id))
    }

    pub fn has_window(&self, id: WindowId) -> bool {
        self.graph.get_window_node(id).is_some()
    }

    pub fn render(&self, config: &Config) {
        let mut pos = WindowPosition::new(0, 0);
        let mut size = WindowSize::new(1920, 1080);

        // if !config.remove_task_bar {
        size.height -= 40;
        // }

        pos.x += config.outer_gap as isize;
        pos.y += config.outer_gap as isize;

        size.width -= config.outer_gap as usize * 2;
        size.height -= config.outer_gap as usize * 2;

        render_node(self.graph.root_node_id, &self.graph, config, pos, size);
    }

    pub fn focus_window(&mut self, id: WindowId) -> WorkspaceResult {
        let node_id = self
            .graph
            .get_window_node(id)
            .ok_or(WorkspaceError::WindowNodeNotFound)?;

        self.focused_node_id = Some(node_id);

        Ok(())
    }

    pub fn focus_in_direction(&mut self, dir: Direction) -> Option<GraphNodeId> {
        self.focused_node_id
            .and_then(|id| self.graph.get_window_node_in_direction(id, dir))
            .map(|node_id| {
                self.focused_node_id = Some(node_id);
                node_id
            })
    }

    pub fn swap_in_direction(&mut self, dir: Direction) -> Option<GraphNodeId> {
        self.focused_node_id
            .and_then(|id| self.graph.get_window_node_in_direction(id, dir))
            .map(|node_id| {
                self.focused_node_id = Some(node_id);
                node_id
            })
    }
}

fn render_node(
    id: GraphNodeId,
    graph: &Graph,
    config: &Config,
    mut pos: WindowPosition,
    mut size: WindowSize,
) {
    let node = graph
        .get_node(id)
        .expect("Cannot render a node that doesn't exist");

    match node {
        GraphNode::Group(kind) => {
            let children = graph.get_children(id);

            if children.len() == 0 {
                return;
            }

            match kind {
                GraphNodeGroupKind::Row => {
                    let col_width = size.width / children.len();
                    let mut x = pos.x;
                    for child_id in children {
                        render_node(
                            child_id,
                            graph,
                            config,
                            WindowPosition::new(x, pos.y),
                            WindowSize::new(col_width, size.height),
                        );
                        x += col_width as isize;
                    }
                }
                GraphNodeGroupKind::Col => {
                    let row_height = size.height / children.len();
                    let mut y = pos.y;
                    for child_id in children {
                        render_node(
                            child_id,
                            graph,
                            config,
                            WindowPosition::new(pos.x, y),
                            WindowSize::new(size.width, row_height),
                        );
                        y += row_height as isize;
                    }
                }
            }
        }
        GraphNode::Window(win_id) => {
            pos.x += config.inner_gap as isize;
            pos.y += config.inner_gap as isize;
            size.width -= config.inner_gap as usize * 2;
            size.height -= config.inner_gap as usize * 2;

            let win = Window::new(*win_id);
            win.reposition(pos);
            win.resize(size);
        }
    }
}