//! Our sessions work similar to vim sessions, in that we don't save sessions automatically
//! ourselves and that we save the layout of each workspace.
//!
//! When loading a session we try to re-manage every window.
//!
//! The format of a session file is as follows:
//!
//! @workspace denotes the start of a workspace section
//! @endworkspace denotes the end of a workspace section
//!
//! A workspace section is split up into two paragraphs (seperated by an empty line).
//!
//! The first paragraph contains a list of nodes, where each node is the serialized version of a
//! `GraphNode`.
//!
//! The format of a serialized node looks like this: `(node id):(node type)[:(window id)]`
//!
//! Example serialized nodes:
//!
//! `1:row`
//! `2:col`
//! `3:win:348024`
//!
//! The second paragraph contains a list of edges, where each edge is the serialized version of a
//! `GraphEdge`.
//!
//! The format of a serialized edge looks like this: `(child node id):(parent node id)`
//!
//! Example serialized edges:
//!
//! `1:2`
//! `3:2`
//! `4:3`
//!
//! TODO: support serializing/deserializing multiple displays

use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::Sender;

use crate::event::Event;
use crate::graph::{Graph, GraphNode, GraphNodeGroupKind, GraphNodeId};
use crate::paths::get_config_path;
use crate::platform::WindowId;
use crate::workspace::{Workspace, WorkspaceId};

use itertools::Itertools;

pub fn save_session(workspaces: &[Workspace]) {
    let session = workspaces
        .iter()
        .map(|workspace| {
            let node_section = workspace
                .graph
                .nodes
                .iter()
                .map(|(node_id, node)| match node {
                    GraphNode::Group(kind) => format!(
                        "{}:{}",
                        node_id,
                        match kind {
                            GraphNodeGroupKind::Row => "row",
                            GraphNodeGroupKind::Col => "col",
                        }
                    ),
                    GraphNode::Window(win_id) => format!("{}:win:{}", node_id, win_id),
                })
                .join("\n");

            let edge_section = workspace
                .graph
                .edges
                .iter()
                .map(|edge| format!("{}:{}", edge.child, edge.parent))
                .join("\n");

            let sections = [node_section, edge_section]
                .iter()
                .filter(|s| !s.is_empty())
                .join("\n\n");

            format!("@workspace {}\n{}\n@endworkspace", workspace.id.0, sections)
        })
        .join("\n");

    let mut path = get_config_path();
    path.push("sessions");

    if !path.exists() {
        fs::create_dir(&path).unwrap();
    }

    path.push("default");

    fs::write(path, session).unwrap();
}

pub fn load_session(tx: Sender<Event>) -> Option<Vec<Workspace>> {
    let mut path = get_config_path();
    path.push("sessions");
    path.push("default");

    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content.split('\n').collect();
    let mut i = 0;

    let mut workspaces = Vec::new();

    while i < lines.len() {
        let line = lines[i];

        if let Some(rest) = line.strip_prefix("@workspace") {
            let id = WorkspaceId(rest.trim().parse::<usize>().ok()?);
            let mut graph = Graph {
                max_id: 0,
                dirty: false,
                root_node_id: 0,
                nodes: HashMap::new(),
                edges: Vec::new(),
            };

            i += 1;

            while i < lines.len() && !lines[i].is_empty() && lines[i] != "@endworkspace" {
                let line = lines[i];
                let parts = line.split(':').collect::<Vec<&str>>();

                let (id, node) = match parts.as_slice() {
                    [node_id, "row"] => (node_id, GraphNode::Group(GraphNodeGroupKind::Row)),
                    [node_id, "col"] => (node_id, GraphNode::Group(GraphNodeGroupKind::Col)),
                    [node_id, "win", win_id] => (
                        node_id,
                        GraphNode::Window(WindowId(win_id.parse::<usize>().unwrap())),
                    ),
                    _ => unreachable!("{:?}", line),
                };

                let id = id.parse::<GraphNodeId>().unwrap();
                if id > graph.max_id {
                    graph.max_id = id;
                }
                graph.nodes.insert(id, node);

                i += 1;
            }

            i += 1;

            while i < lines.len() && lines[i] != "@endworkspace" {
                let line = lines[i];
                let parts = line.split(':').collect::<Vec<&str>>();

                let child = parts[0].parse::<GraphNodeId>().unwrap();
                let parent = parts[1].parse::<GraphNodeId>().unwrap();

                graph.add_edge(parent, child);

                i += 1;
            }

            let mut workspace = Workspace::new(id, tx.clone());
            workspace.graph = graph;
            workspaces.push(workspace);
        }

        i += 1;
    }

    Some(workspaces)
}
