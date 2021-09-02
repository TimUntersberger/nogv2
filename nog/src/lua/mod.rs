pub mod config_proxy;
pub mod conversions;
pub mod graph_proxy;
pub mod namespace;
pub mod repl;
pub mod runtime;

use std::{sync::{Arc, RwLock, mpsc::Sender}};

pub use namespace::LuaNamespace;
pub use runtime::LuaRuntime;

use mlua::prelude::*;
use std::str::FromStr;

use crate::{direction::Direction, event::{Action, Event, WindowAction, WorkspaceAction}, key_combination::KeyCombination, keybinding::KeybindingMode, lua::config_proxy::ConfigProxy, paths::{get_runtime_path, get_config_path}, platform::{NativeWindow, Window, WindowId}, window_manager::WindowManager, workspace::WorkspaceId};

pub fn init<'a>(tx: Sender<Event>, wm: Arc<RwLock<WindowManager>>) -> LuaResult<LuaRuntime<'a>> {
    let rt = LuaRuntime::new(tx.clone(), wm.clone())?;

    rt.namespace
        .add_constant("runtime_path", get_runtime_path().to_str().unwrap())?;

    rt.namespace
        .add_constant("config_path", get_config_path().to_str().unwrap())?;

    rt.namespace
        .add_constant("version", option_env!("NOG_VERSION").unwrap_or("DEV"))?;

    rt.namespace.add_function(
        "bind",
        |tx, _wm, lua, (mode, key_combination, cb): (KeybindingMode, KeyCombination, mlua::Function)| {
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
        .add_function("unbind", |tx, _wm, _lua, key: String| {
            tx.send(Event::Action(Action::RemoveKeybinding { key }))
                .unwrap();
            Ok(())
        })?;

    rt.namespace
        .add_function("update_window_layout", |tx, _wm, _lua, (): ()| {
            tx.send(Event::RenderGraph).unwrap();
            Ok(())
        })?;

    rt.namespace.add_function(
        "ws_focus",
        |tx, _wm, _lua, (ws_id, direction): (Option<WorkspaceId>, Direction)| {
            tx.send(Event::Action(Action::Workspace(WorkspaceAction::Focus(
                ws_id, direction,
            ))))
            .unwrap();
            Ok(())
        },
    )?;

    rt.namespace.add_function(
        "ws_swap",
        |tx, _wm, _lua, (ws_id, direction): (Option<WorkspaceId>, Direction)| {
            tx.send(Event::Action(Action::Workspace(WorkspaceAction::Swap(
                ws_id, direction,
            ))))
            .unwrap();
            Ok(())
        },
    )?;

    rt.namespace
        .add_function("session_save", |tx, _wm, _lua, name: Option<String>| {
            tx.send(Event::Action(Action::SaveSession)).unwrap();
            Ok(())
        })?;

    rt.namespace
        .add_function("session_load", |tx, _wm, _lua, name: Option<String>| {
            tx.send(Event::Action(Action::LoadSession)).unwrap();
            Ok(())
        })?;

    rt.namespace
        .add_function("win_close", |tx, _wm, _lua, win_id: Option<WindowId>| {
            tx.send(Event::Action(Action::Window(WindowAction::Close(win_id))))
                .unwrap();
            Ok(())
        })?;

    rt.namespace
        .add_function("win_is_managed", |tx, wm, _lua, win_id: Option<WindowId>| {
            let id = win_id.unwrap_or_else(|| Window::get_foreground_window().get_id());
            Ok(wm.read().unwrap().is_window_managed(id))
        })?;

    rt.namespace
        .add_function("win_manage", |tx, _wm, _lua, win_id: Option<WindowId>| {
            tx.send(Event::Action(Action::Window(WindowAction::Manage(win_id))))
                .unwrap();
            Ok(())
        })?;

    rt.namespace
        .add_function("win_unmanage", |tx, _wm, _lua, win_id: Option<WindowId>| {
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
