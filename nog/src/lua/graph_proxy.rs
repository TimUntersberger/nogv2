use mlua::prelude::*;

pub struct GraphProxy;

impl mlua::UserData for GraphProxy {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {}
}
