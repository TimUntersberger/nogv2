pub mod config_proxy;
pub mod conversions;
pub mod graph_proxy;
pub mod namespace;
pub mod repl;
pub mod runtime;

use std::sync::{mpsc::Sender, Arc, RwLock};

pub use namespace::LuaNamespace;
pub use runtime::LuaRuntime;

use mlua::prelude::*;
use std::str::FromStr;

use crate::{
    action::{Action, WindowAction, WorkspaceAction},
    direction::Direction,
    display::DisplayId,
    event::Event,
    key_combination::KeyCombination,
    keybinding::KeybindingMode,
    lua::config_proxy::ConfigProxy,
    paths::{get_config_path, get_runtime_path},
    platform::{Api, NativeApi, NativeWindow, WindowId},
    types::ThreadSafeWindowManagers,
    window_manager::WindowManager,
    workspace::WorkspaceId,
};

struct BarLayout<'a> {
    left: mlua::Table<'a>,
    center: mlua::Table<'a>,
    right: mlua::Table<'a>,
}

impl<'lua> FromLua<'lua> for BarLayout<'lua> {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let tbl = mlua::Table::from_lua(lua_value, lua)?;
        let left = tbl.get("left").or_else(|_| lua.create_table())?;
        let center = tbl.get("center").or_else(|_| lua.create_table())?;
        let right = tbl.get("right").or_else(|_| lua.create_table())?;

        Ok(BarLayout {
            left,
            center,
            right,
        })
    }
}

macro_rules! inject_mapper {
    ($tx:ident, $wms:ident, $lua:ident, lua) => {
        $lua
    };
    ($tx:ident, $wms:ident, $lua:ident, tx) => {
        $tx
    };
    ($tx:ident, $wms:ident, $lua:ident, wms) => {
        $wms
    };
}

macro_rules! namespace {
    ($rt:ident, {
        $(const $const_ident:ident = $expr:expr);*;
        $(fn $fn_ident:ident ($($arg_n:ident : $arg_t:ty),*) {
            inject $($inject_ident:ident),*;
            $($body:tt)*
        });*;
    }) => {
        $($rt.namespace
            .add_constant(stringify!($const_ident), $expr)?;)*

        #[allow(unused_parens)]
        {
            $($rt.namespace
                .add_function(stringify!($fn_ident), |_tx, _wms, _lua, ($($arg_n),*): ($($arg_t),*)| {
                    $(let $inject_ident = inject_mapper!(_tx, _wms, _lua, $inject_ident);)*
                    $($body)*
                })?;)*
        }
    };
}

pub fn init<'a>(tx: Sender<Event>, wms: ThreadSafeWindowManagers) -> LuaResult<LuaRuntime<'a>> {
    let rt = LuaRuntime::new(tx.clone(), wms.clone())?;

    namespace!(rt, {
        const runtime_path = get_runtime_path().to_str().unwrap();
        const config_path = get_config_path().to_str().unwrap();
        const version = option_env!("NOG_VERSION").unwrap_or("DEV");
        const config = ConfigProxy::new(tx);

        fn bind(mode: KeybindingMode, key_combination: KeyCombination, cb: mlua::Function) {
            inject lua, tx;

            lua.set_named_registry_value(&key_combination.get_id().to_string(), cb)?;
            tx.send(Event::Action(Action::CreateKeybinding {
                mode,
                key_combination,
            }))
            .unwrap();
            Ok(())
        };

        fn unbind(key: String) {
            inject tx;

            tx.send(Event::Action(Action::RemoveKeybinding { key }))
                .unwrap();
            Ok(())
        };

        fn update_window_layout() {
            inject tx;

            tx.send(Event::RenderGraph).unwrap();

            Ok(())
        };

        fn bar_set_layout(layout: BarLayout) {
            inject lua, tx;

            lua.set_named_registry_value("left", layout.left)?;
            lua.set_named_registry_value("center", layout.center)?;
            lua.set_named_registry_value("right", layout.right)?;
            tx.send(Event::RenderBarLayout).unwrap();

            Ok(())
        };

        fn exit() {
            inject tx;

            tx.send(Event::Exit).unwrap();

            Ok(())
        };

        fn ws_focus(ws_id: Option<WorkspaceId>, direction: Direction) {
            inject tx;

            tx.send(Event::Action(Action::Workspace(WorkspaceAction::Focus(
                ws_id, direction,
            ))))
            .unwrap();

            Ok(())
        };

        fn ws_get_all() {
            inject wms;

            let mut workspaces = vec![];

            for wm in wms.read().unwrap().iter() {
                let ws_ids = wm
                    .read()
                    .unwrap()
                    .workspaces
                    .iter()
                    .map(|w| w.id)
                    .collect::<Vec<_>>();

                for id in ws_ids {
                    workspaces.push(id);
                }
            }

            Ok(workspaces)
        };

        fn ws_swap(ws_id: Option<WorkspaceId>, direction: Direction) {
            inject tx;

            tx.send(Event::Action(Action::Workspace(WorkspaceAction::Swap(
                ws_id, direction,
            ))))
            .unwrap();

            Ok(())
        };

        fn session_save() {
            inject tx;

            tx.send(Event::Action(Action::SaveSession)).unwrap();

            Ok(())
        };

        fn session_load() {
            inject tx;

            tx.send(Event::Action(Action::LoadSession)).unwrap();

            Ok(())
        };

        fn win_close(win_id: Option<WindowId>) {
            inject tx;

            tx.send(Event::Action(Action::Window(WindowAction::Close(win_id))))
                .unwrap();

            Ok(())
        };

        fn win_is_managed(win_id: Option<WindowId>) {
            inject wms;

            let id = win_id.unwrap_or_else(|| Api::get_foreground_window().get_id());
            let wms = wms.read().unwrap();

            Ok(wms.iter().any(|wm| wm.read().unwrap().has_window(id)))
        };

        fn win_is_managed(win_id: Option<WindowId>) {
            inject tx;

            tx.send(Event::Action(Action::Window(WindowAction::Manage(win_id))))
                .unwrap();

            Ok(())
        };

        fn win_unmanage(win_id: Option<WindowId>) {
            inject tx;

            tx.send(Event::Action(Action::Window(WindowAction::Unmanage(
                win_id,
            ))))
            .unwrap();

            Ok(())
        };

        fn dsp_contains_ws(dsp_id: Option<DisplayId>, ws_id: WorkspaceId) {
            inject wms;

            todo!();
            //wm.read().unwrap().display_id =

            Ok(())
        };
    });

    rt.namespace.register(None)?;

    // Run the nog init.lua
    rt.eval("dofile(nog.runtime_path .. '/lua/init.lua')")?;

    Ok(rt)
}
