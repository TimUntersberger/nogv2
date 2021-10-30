use crate::state::State;

use super::LuaNamespace;
use mlua::prelude::*;

pub struct LuaRuntime {
    /// LuaRuntime contains a copy of the state for ease of use
    state: State,
    pub lua: &'static Lua,
}

impl LuaRuntime {
    pub fn new(state: State) -> LuaResult<Self> {
        let options = LuaOptions::default();
        let lua = unsafe { Lua::unsafe_new_with(mlua::StdLib::ALL, options) }.into_static();
        Ok(Self { state, lua })
    }

    pub fn eval(&self, s: &str) -> LuaResult<mlua::Value> {
        self.lua.load(s).eval()
    }

    pub fn create_namespace(&self, s: &str) -> LuaResult<LuaNamespace> {
        LuaNamespace::new(self.state.clone(), self.lua, s)
    }

    pub fn call_fn<'a, A>(&'a self, path: &str, args: A) -> LuaResult<mlua::Value>
    where
        A: ToLuaMulti<'a>,
    {
        mlua::Function::from_lua(self.eval(path)?, self.lua)?.call(args)
    }
}
