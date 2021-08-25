use mlua::prelude::*;
use super::LuaNamespace;

pub struct LuaRuntime<'a> {
    pub rt: &'static Lua,
    pub namespace: LuaNamespace<'a>,
}

impl<'a> LuaRuntime<'a> {
    pub fn new() -> LuaResult<Self> {
        let rt = unsafe { Lua::unsafe_new() }.into_static();
        Ok(Self {
            rt,
            namespace: LuaNamespace::new(&rt, "nog")?,
        })
    }

    pub fn eval(&self, s: &str) -> LuaResult<mlua::Value> {
        self.rt.load(s).eval()
    }
}
