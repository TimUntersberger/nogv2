use std::collections::HashMap;

use crate::direction::Direction;
use crate::platform::WindowId;

pub type WindowNodeId = WindowId;
pub type GraphNodeId = usize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphNodeGroupKind {
    Row,
    Col,
}

#[derive(Debug, Clone)]
pub enum GraphNode {
    Group(GraphNodeGroupKind),
    Window(WindowNodeId),
}

impl GraphNode {
    pub fn try_get_window_id(&self) -> Option<WindowNodeId> {
        match self {
            GraphNode::Window(win_id) => Some(*win_id),
            _ => None,
        }
    }

    pub fn try_get_group_kind(&self) -> Option<GraphNodeGroupKind> {
        match self {
            GraphNode::Group(kind) => Some(*kind),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GraphError {
    /// The node is not a group node.
    NotAGroupNode,
    NodeNotFound,
}

pub type GraphResult<T = ()> = Result<T, GraphError>;

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub parent: GraphNodeId,
    pub child: GraphNodeId,
}

#[derive(Debug, Clone)]
pub struct Graph {
    // Holds the biggest id that has been given out.
    // Gets increased whenever a new node gets added.
    //
    // So basically very primitive id generation
    pub max_id: GraphNodeId,
    // Whether the graph has been modified and not yet handled
    pub dirty: bool,
    pub nodes: HashMap<GraphNodeId, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub root_node_id: GraphNodeId,
}

impl Graph {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(0, GraphNode::Group(GraphNodeGroupKind::Row));
        Self {
            max_id: 0,
            dirty: false,
            root_node_id: 0,
            nodes,
            edges: vec![],
        }
    }

    fn add_child_node(
        &mut self,
        parent_id: GraphNodeId,
        child: GraphNode,
    ) -> GraphResult<GraphNodeId> {
        if let Some(GraphNode::Group(_)) = self.nodes.get(&parent_id) {
            self.max_id += 1;
            self.nodes.insert(self.max_id, child);
            self.add_edge(parent_id, self.max_id);
            self.dirty = true;
            return Ok(self.max_id);
        }

        Err(GraphError::NotAGroupNode)
    }

    pub fn add_edge(&mut self, parent: GraphNodeId, child: GraphNodeId) {
        self.dirty = true;
        self.edges.push(GraphEdge { parent, child });
    }

    pub fn add_row(&mut self, parent_id: GraphNodeId) -> GraphResult<GraphNodeId> {
        self.add_child_node(parent_id, GraphNode::Group(GraphNodeGroupKind::Row))
    }

    pub fn add_col(&mut self, parent_id: GraphNodeId) -> GraphResult<GraphNodeId> {
        self.add_child_node(parent_id, GraphNode::Group(GraphNodeGroupKind::Col))
    }

    pub fn add_window(
        &mut self,
        parent_id: GraphNodeId,
        window_id: WindowNodeId,
    ) -> GraphResult<GraphNodeId> {
        self.add_child_node(parent_id, GraphNode::Window(window_id))
    }

    pub fn get_node_mut(&mut self, id: GraphNodeId) -> Option<&mut GraphNode> {
        self.nodes.get_mut(&id)
    }

    pub fn get_node(&self, id: GraphNodeId) -> Option<&GraphNode> {
        self.nodes.get(&id)
    }

    pub fn get_window_node(&self, win_id: WindowNodeId) -> Option<GraphNodeId> {
        self.nodes
            .iter()
            .find(|(_, node)| match node {
                GraphNode::Group(_) => false,
                GraphNode::Window(id) => *id == win_id,
            })
            .map(|(id, _)| *id)
    }

    pub fn get_parent_node(&self, child: GraphNodeId) -> Option<GraphNodeId> {
        self.get_parent_edge(child)
            .map(|edge| self.edges[edge].parent)
    }

    /// Traverses the node tree down until it finds a window node.
    ///
    /// Example A:
    ///
    /// Row
    /// |  Col
    /// |  |  Win(1)
    ///
    /// -> Win(1)
    ///
    /// Example B:
    ///
    /// Row
    /// |  Col
    /// |  |  Win(1)
    /// |  Win(2)
    ///
    /// -> Win(1)
    pub fn get_first_window_child(&self, id: GraphNodeId) -> Option<GraphNodeId> {
        match self.get_node(id).and_then(|node| node.try_get_group_kind()) {
            // It doesn't matter whether it is a row or column
            Some(_) => {
                let children = self.get_children(id);
                self.get_first_window_child(children[0])
            }
            None => Some(id),
        }
    }

    /// The start node is usually the currently focused node of a workspace
    pub fn get_window_node_in_direction(
        &self,
        start: GraphNodeId,
        dir: Direction,
    ) -> Option<GraphNodeId> {
        let parent_node_id = self
            .get_parent_node(start)
            .expect("The focused node has to have a parent node");

        let parent_node_group_kind = self
            .get_node(parent_node_id)
            .unwrap()
            .try_get_group_kind()
            .expect("The parent node has to be a group node");

        let target_group_kind = match dir {
            Direction::Left | Direction::Right => GraphNodeGroupKind::Row,
            Direction::Up | Direction::Down => GraphNodeGroupKind::Col,
        };

        // The container_parent is above the parent of the focused node
        //
        // Row (container_parent)
        // |  Win
        // |  Col (parent)
        // |  |  Win (node)
        //
        // We can assume that the container_parent is a row, because we
        // can't have nested columns
        let (parent_id, child_id) = match parent_node_group_kind {
            k if k == target_group_kind => (parent_node_id, start),
            _ if parent_node_id != self.root_node_id => (
                self.get_parent_node(parent_node_id)
                    .expect("Can't be root node"),
                parent_node_id,
            ),
            _ => return None,
        };

        let children = self.get_children(parent_id);
        let idx = children
            .iter()
            .enumerate()
            .find(|(_, c)| **c == child_id)
            .map(|(idx, _)| idx)
            .expect("The parent of a node has to have the node as its child");

        let target_idx: isize = match dir {
            Direction::Left | Direction::Up => idx as isize - 1,
            Direction::Right | Direction::Down => idx as isize + 1,
        };

        if target_idx < 0 || target_idx >= children.len() as isize {
            None
        } else {
            self.get_first_window_child(children[target_idx as usize])
        }
    }

    pub fn get_children(&self, parent: GraphNodeId) -> Vec<GraphNodeId> {
        self.edges
            .iter()
            .filter(|e| e.parent == parent)
            .map(|e| e.child)
            .collect()
    }

    // A parent edge is where the child has the given id
    pub fn get_parent_edge(&self, id: GraphNodeId) -> Option<usize> {
        self.edges
            .iter()
            .enumerate()
            .find(|(_, e)| e.child == id)
            .map(|(idx, _)| idx)
    }

    pub fn delete_node(&mut self, id: GraphNodeId) -> GraphResult {
        if let Some(_) = self.nodes.remove(&id) {
            self.dirty = true;

            if self.max_id == id {
                self.max_id -= 1;
            }

            self.edges
                .remove(self.get_parent_edge(id).ok_or(GraphError::NodeNotFound)?);

            for c in self.get_children(id) {
                self.delete_node(c)?;
            }

            Ok(())
        } else {
            Err(GraphError::NodeNotFound)
        }
    }

    pub fn move_node(&mut self, new_parent: GraphNodeId, node: GraphNodeId, index: Option<usize>) {
        let parent_edge_idx = self
            .get_parent_edge(node)
            .expect("You cannot move the root node");

        self.edges.remove(parent_edge_idx);

        if let Some(index) = index {
            if index == 0 {
                self.dirty = true;
                self.edges.insert(
                    0,
                    GraphEdge {
                        parent: new_parent,
                        child: node,
                    },
                );
            } else {
                let mut count = 0;
                let mut idx = 0;

                for (edge_idx, edge) in self.edges.iter().enumerate() {
                    if edge.parent == new_parent {
                        count += 1;
                        if count > index {
                            idx = edge_idx;
                            break;
                        }
                    }
                }

                self.dirty = true;
                self.edges.insert(
                    idx,
                    GraphEdge {
                        parent: new_parent,
                        child: node,
                    },
                );
            }
        } else {
            self.add_edge(new_parent, node);
        }
    }

    pub fn swap_nodes(&mut self, x: GraphNodeId, y: GraphNodeId) {
        for edge in &mut self.edges {
            if edge.child == x {
                edge.child = y;
            } else if edge.child == y {
                edge.child = x;
            }
        }

        self.dirty = true;
    }
}
