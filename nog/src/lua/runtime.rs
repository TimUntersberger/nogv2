use std::collections::HashMap;

use super::LuaNamespace;
use mlua::prelude::*;

pub struct LuaRuntime<'a> {
    pub rt: &'static Lua,
    pub namespace: LuaNamespace<'a>,
    /// Holds the latest callback id. This is only increases and is meant as a primitive auto
    /// increment id.
    latest_callback_id: u32,
    callback_store: HashMap<u32, mlua::Function<'a>>
}

impl<'a> LuaRuntime<'a> {
    pub fn new() -> LuaResult<Self> {
        let options = LuaOptions::default();
        let rt = unsafe { Lua::unsafe_new_with(mlua::StdLib::ALL, options) }.into_static();
        Ok(Self {
            rt,
            namespace: LuaNamespace::new(&rt, "nog")?,
            latest_callback_id: 0,
            callback_store: HashMap::new()
        })
    }

    pub fn eval(&self, s: &str) -> LuaResult<mlua::Value> {
        self.rt.load(s).eval()
    }

    pub fn add_callback(&mut self, f: mlua::Function<'a>) {
        self.latest_callback_id += 1;
        self.callback_store.insert(1, f);
    }

    pub fn remove_callback(&mut self, id: u32) {
        self.callback_store.remove(&id);
    }

    pub fn create_namespace(&self, s: &str) -> LuaResult<LuaNamespace<'a>> {
        LuaNamespace::new(self.rt, s)
    }
}
