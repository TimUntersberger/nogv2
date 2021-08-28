use std::collections::HashMap;

use crate::platform::WindowId;

pub type WindowNodeId = WindowId;
pub type GraphNodeId = usize;

#[derive(Debug, Clone)]
pub enum GraphNodeGroupKind {
    Row,
    Col,
}

#[derive(Debug, Clone)]
pub enum GraphNode {
    Group(GraphNodeGroupKind),
    Window(WindowNodeId),
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
    parent: GraphNodeId,
    child: GraphNodeId,
}

#[derive(Debug, Clone)]
pub struct Graph {
    // Holds the biggest id that has been given out.
    // Gets increased whenever a new node gets added.
    //
    // So basically very primitive id generation
    max_id: GraphNodeId,
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
