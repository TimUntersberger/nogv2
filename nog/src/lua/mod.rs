mod config_proxy;
mod conversions;
mod namespace;
pub mod repl;
mod runtime;

use std::{path::PathBuf, sync::{Arc, mpsc::Sender}};

pub use namespace::LuaNamespace;
pub use runtime::LuaRuntime;

use mlua::prelude::*;
use std::str::FromStr;

use crate::{keybinding::KeybindingMode, event::{Action, Event}, lua::config_proxy::ConfigProxy};

fn get_runtime_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        let mut path: PathBuf = std::env::current_exe().unwrap();
        path.pop();
        path.pop();
        path.pop();
        path.push("nog");
        path.push("runtime");
        path
    }
    #[cfg(not(debug_assertions))]
    {
        let mut path: PathBuf = dirs::data_dir().unwrap_or_default();
        path.push("nog");
        path.push("runtime");
        path
    }
}

pub fn init<'a>(tx: Sender<Event>) -> LuaResult<LuaRuntime<'a>> {
    let rt = LuaRuntime::new(tx.clone())?;

    rt.namespace
        .add_constant("runtime_path", get_runtime_path().to_str().unwrap())?;

    rt.namespace
        .add_constant("version", option_env!("NOG_VERSION").unwrap_or("DEV"))?;

    rt.namespace.add_function("bind", |tx, _lua, (mode, key, cb): (KeybindingMode, String, mlua::Function)| {
        tx.send(Event::Action(Action::CreateKeybinding {
            mode,
            key
        })).unwrap();
        Ok(())
    })?;

    rt.namespace.add_function("unbind", |tx, _lua, key: String| {
        tx.send(Event::Action(Action::RemoveKeybinding {
            key
        })).unwrap();
        Ok(())
    })?;

    rt.namespace.add_constant("config", ConfigProxy::new(tx))?;

    rt.namespace.register(None)?;

    // Run the nog init.lua
    rt.eval("dofile(nog.runtime_path .. '/lua/init.lua')")?;

    Ok(rt)
}
