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
    Group {
        kind: GraphNodeGroupKind,
        /// The index of the child which has focus
        ///
        /// NOTE: NOT the index for the nodes vec in the graph
        ///
        /// Example: If the focus is `2` it doesn't mean that the node with id `2` has focus, but
        /// that the second child of this group has focus.
        focus: usize,
        /// How many nodes are connected as children to this group node
        child_count: usize,
    },
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
            GraphNode::Group { kind, .. } => Some(*kind),
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

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();

        nodes.insert(
            0,
            GraphNode::Group {
                kind: GraphNodeGroupKind::Row,
                focus: 0,
                child_count: 0,
            },
        );

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
        if let Some(GraphNode::Group {
            focus, child_count, ..
        }) = self.nodes.get_mut(&parent_id)
        {
            *child_count += 1;
            *focus = *child_count - 1;

            self.max_id += 1;
            self.nodes.insert(self.max_id, child);
            self.add_edge(parent_id, self.max_id);

            self.dirty = true;

            return Ok(self.max_id);
        }

        Err(GraphError::NotAGroupNode)
    }

    /// WARNING: this function DOES NOT update the `child_count` and `focus` of the parent node.
    pub fn add_edge(&mut self, parent: GraphNodeId, child: GraphNodeId) {
        self.dirty = true;
        self.edges.push(GraphEdge { parent, child });
    }

    pub fn add_row(&mut self, parent_id: GraphNodeId) -> GraphResult<GraphNodeId> {
        self.add_child_node(
            parent_id,
            GraphNode::Group {
                kind: GraphNodeGroupKind::Row,
                child_count: 0,
                focus: 0,
            },
        )
    }

    pub fn add_col(&mut self, parent_id: GraphNodeId) -> GraphResult<GraphNodeId> {
        self.add_child_node(
            parent_id,
            GraphNode::Group {
                kind: GraphNodeGroupKind::Col,
                child_count: 0,
                focus: 0,
            },
        )
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
                GraphNode::Group { .. } => false,
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
    pub fn get_focused_window_child(&self, id: GraphNodeId) -> Option<GraphNodeId> {
        match self.get_node(id) {
            // It doesn't matter whether it is a row or column
            Some(GraphNode::Group { focus, .. }) => {
                let children = self.get_children(id);
                children.get(*focus).and_then(|x| self.get_focused_window_child(*x))
            }
            _ => Some(id),
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
        // We can assume that the container_parent is the opposite group kind, because we
        // can't have nested groups of the same kind.
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
            self.get_focused_window_child(children[target_idx as usize])
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

    /// If shallow is set to true, then this function won't delete the children of a group node.
    pub fn delete_node(&mut self, id: GraphNodeId, shallow: bool) -> GraphResult {
        if self.nodes.remove(&id).is_some() {
            self.dirty = true;

            if self.max_id == id {
                self.max_id -= 1;
            }

            let parent_edge_idx = self.get_parent_edge(id).ok_or(GraphError::NodeNotFound)?;
            let parent = self
                .get_node_mut(self.edges[parent_edge_idx].parent)
                .unwrap();

            match parent {
                GraphNode::Group {
                    focus, child_count, ..
                } => {
                    *focus = (*focus).max(1) - 1;
                    *child_count -= 1;
                }
                _ => unreachable!(),
            }

            self.edges.remove(parent_edge_idx);

            if !shallow {
                for c in self.get_children(id) {
                    self.delete_node(c, false)?;
                }
            }

            Ok(())
        } else {
            Err(GraphError::NodeNotFound)
        }
    }

    pub fn move_node(&mut self, new_parent: GraphNodeId, node: GraphNodeId) {
        let node_cpy = self.get_node(node).unwrap().clone();

        self.delete_node(node, true).unwrap();
        self.add_child_node(new_parent, node_cpy).unwrap();
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

fn node_to_string(depth: usize, id: GraphNodeId, graph: &Graph) -> Vec<String> {
    let node = graph
        .get_node(id)
        .expect("Cannot print a node that doesn't exist");

    let indent = "|   ".repeat(depth);

    match node {
        GraphNode::Group {
            kind,
            focus,
            child_count,
        } => {
            let children = graph.get_children(id);

            let tag = match kind {
                GraphNodeGroupKind::Row => "Row",
                GraphNodeGroupKind::Col => "Col",
            };

            let mut s = vec![format!(
                "{}[{}]{} focus: {} child_count: {}",
                indent, id, tag, focus, child_count
            )];

            for child_id in children {
                s.append(&mut node_to_string(depth + 1, child_id, graph));
            }

            s
        }
        GraphNode::Window(win_id) => {
            vec![format!("{}[{}]Win({})", indent, id, win_id)]
        }
    }
}

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            node_to_string(0, self.root_node_id, self).join("\n")
        )
    }
}
