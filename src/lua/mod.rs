mod namespace;
mod runtime;

pub use namespace::LuaNamespace;
pub use runtime::LuaRuntime;

use mlua::prelude::*;

pub fn init<'a>() -> LuaResult<LuaRuntime<'a>> {
    let mut rt = LuaRuntime::new()?;

    rt.namespace
        .add_function("say", |(name, age, msg): (String, i32, String)| {
            dbg!(name, age, msg);
        })?;

    rt.namespace
        .add_namespace(LuaNamespace::new(&rt.rt, "temp")?);

    rt.namespace.register(None)?;

    Ok(rt)
}
