use crate::direction::Direction;
use crate::event::{Action, Event, WindowAction};
use crate::graph::{Graph, GraphNode, GraphNodeGroupKind, GraphNodeId};
use crate::platform::WindowId;
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
