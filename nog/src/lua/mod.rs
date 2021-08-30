pub mod config_proxy;
pub mod conversions;
pub mod graph_proxy;
pub mod namespace;
pub mod repl;
pub mod runtime;

use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc},
};

pub use namespace::LuaNamespace;
pub use runtime::LuaRuntime;

use mlua::prelude::*;
use std::str::FromStr;

use crate::{
    direction::Direction,
    event::{Action, Event, WindowAction, WorkspaceAction},
    key_combination::KeyCombination,
    keybinding::KeybindingMode,
    lua::config_proxy::ConfigProxy,
    platform::WindowId,
    workspace::WorkspaceId,
};

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

    rt.namespace.add_function(
        "bind",
        |tx, lua, (mode, key_combination, cb): (KeybindingMode, KeyCombination, mlua::Function)| {
            lua.set_named_registry_value(&key_combination.get_id().to_string(), cb)?;
            tx.send(Event::Action(Action::CreateKeybinding {
                mode,
                key_combination,
            }))
            .unwrap();
            Ok(())
        },
    )?;

    rt.namespace
        .add_function("unbind", |tx, _lua, key: String| {
            tx.send(Event::Action(Action::RemoveKeybinding { key }))
                .unwrap();
            Ok(())
        })?;

    rt.namespace
        .add_function("update_window_layout", |tx, _lua, (): ()| {
            tx.send(Event::RenderGraph).unwrap();
            Ok(())
        })?;

    rt.namespace.add_function(
        "ws_focus",
        |tx, _lua, (ws_id, direction): (Option<WorkspaceId>, Direction)| {
            tx.send(Event::Action(Action::Workspace(WorkspaceAction::Focus(
                ws_id, direction,
            ))))
            .unwrap();
            Ok(())
        },
    )?;

    rt.namespace.add_function(
        "ws_swap",
        |tx, _lua, (ws_id, direction): (Option<WorkspaceId>, Direction)| {
            tx.send(Event::Action(Action::Workspace(WorkspaceAction::Swap(
                ws_id, direction,
            ))))
            .unwrap();
            Ok(())
        },
    )?;

    rt.namespace
        .add_function("win_close", |tx, _lua, win_id: Option<WindowId>| {
            tx.send(Event::Action(Action::Window(WindowAction::Close(win_id))))
                .unwrap();
            Ok(())
        })?;

    rt.namespace
        .add_function("win_manage", |tx, _lua, win_id: Option<WindowId>| {
            tx.send(Event::Action(Action::Window(WindowAction::Manage(win_id))))
                .unwrap();
            Ok(())
        })?;

    rt.namespace
        .add_function("win_unmanage", |tx, _lua, win_id: Option<WindowId>| {
            tx.send(Event::Action(Action::Window(WindowAction::Unmanage(
                win_id,
            ))))
            .unwrap();
            Ok(())
        })?;

    rt.namespace.add_constant("config", ConfigProxy::new(tx))?;

    rt.namespace.register(None)?;

    // Run the nog init.lua
    rt.eval("dofile(nog.runtime_path .. '/lua/init.lua')")?;

    Ok(rt)
}
