use mlua::prelude::*;

use crate::{
    direction::Direction,
    graph::{Graph, GraphNode, GraphNodeId, WindowNodeId},
};

pub struct GraphProxy<'a>(pub &'a mut Graph);

impl<'a> mlua::UserData for GraphProxy<'a> {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "add_window_node",
            |_lua, this, (parent_id, win_id): (Option<GraphNodeId>, WindowNodeId)| {
                let parent_id = parent_id.unwrap_or(this.0.root_node_id);
                Ok(this.0.add_window(parent_id, win_id).ok())
            },
        );

        methods.add_method_mut(
            "add_column_node",
            |_lua, this, parent_id: Option<GraphNodeId>| {
                let parent_id = parent_id.unwrap_or(this.0.root_node_id);
                Ok(this.0.add_col(parent_id).ok())
            },
        );

        methods.add_method_mut(
            "add_row_node",
            |_lua, this, parent_id: Option<GraphNodeId>| {
                let parent_id = parent_id.unwrap_or(this.0.root_node_id);
                Ok(this.0.add_row(parent_id).ok())
            },
        );

        methods.add_method_mut("del_node", |_lua, this, node: GraphNodeId| {
            this.0.delete_node(node).ok();
            Ok(())
        });

        methods.add_method_mut(
            "swap_nodes",
            |_lua, this, (x, y): (GraphNodeId, GraphNodeId)| {
                this.0.swap_nodes(x, y);
                Ok(())
            },
        );

        methods.add_method_mut(
            "move_node",
            |_lua, this, (parent_id, node_id, index): (Option<GraphNodeId>, GraphNodeId, Option<usize>)| {
                let parent_id = parent_id.unwrap_or(this.0.root_node_id);
                Ok(this.0.move_node(parent_id, node_id, index))
            },
        );

        methods.add_method(
            "get_window_node_in_direction",
            |_lua, this, (start, direction): (GraphNodeId, Direction)| {
                Ok(this.0.get_window_node_in_direction(start, direction))
            },
        );

        methods.add_method_mut("del_window_node", |_lua, this, (win_id): (WindowNodeId)| {
            let maybe_node_id = this.0.get_window_node(win_id);

            if let Some(node_id) = maybe_node_id {
                if let Ok(_) = this.0.delete_node(node_id) {
                    return Ok(Some(node_id));
                }
            }

            Ok(None)
        });
    }
}
