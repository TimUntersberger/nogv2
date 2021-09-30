pub mod config_proxy;
pub mod conversions;
pub mod graph_proxy;
pub mod namespace;
pub mod runtime;

pub use namespace::LuaNamespace;
pub use runtime::LuaRuntime;

use mlua::prelude::*;

use crate::{
    action::{Action, WindowAction, WorkspaceAction},
    direction::Direction,
    display::DisplayId,
    event::Event,
    key_combination::KeyCombination,
    keybinding::KeybindingMode,
    lua::config_proxy::ConfigProxy,
    paths::{get_config_path, get_runtime_path},
    platform::{Api, NativeApi, NativeWindow, Window, WindowId},
    rgb::Rgb,
    state::State,
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
    ($state:ident, $lua:ident, lua) => {
        $lua
    };
    ($state:ident, $lua:ident, state) => {
        $state
    };
}

macro_rules! namespace {
    ($rt:ident, $ns_name:ident, {
        $(const $const_ident:ident = $expr:expr);*;
        $(fn $fn_ident:ident ($($arg_n:ident : $arg_t:ty),*) {
            inject $($inject_ident:ident),*;
            $($body:tt)*
        })*
    }) => {
        {
            let ns = $rt.create_namespace(stringify!($ns_name))?;
            $(ns.add_constant(stringify!($const_ident), $expr)?;)*

            #[allow(unused_parens)]
            {
                $(ns.add_function(stringify!($fn_ident), |_state, _lua, ($($arg_n),*): ($($arg_t),*)| {
                    $(let $inject_ident = inject_mapper!(_state, _lua, $inject_ident);)*
                    $($body)*
                })?;)*
            }

            ns
        }
    };
}

pub fn init(state: State) -> LuaResult<LuaRuntime> {
    let rt = LuaRuntime::new(state.clone())?;

    let ns = namespace!(rt, nog, {
        const runtime_path = get_runtime_path().to_str().unwrap();
        const config_path = get_config_path().to_str().unwrap();
        const version = option_env!("NOG_VERSION").unwrap_or("DEV");
        const config = ConfigProxy::new(state.tx.clone(), state.config);

        fn scale_color(hex: i32, factor: f32) {
            inject state;

            Ok(Rgb::from_hex(hex).scaled(factor))
        }

        fn bind(mode: KeybindingMode, key_combination: KeyCombination, cb: mlua::Function) {
            inject lua, state;

            lua.set_named_registry_value(&key_combination.get_id().to_string(), cb)?;
            state.tx.send(Event::Action(Action::CreateKeybinding {
                mode,
                key_combination,
            }))
            .unwrap();
            Ok(())
        }

        fn simulate_key_press(kc: KeyCombination) {
            inject state;

            state.tx.send(Event::Action(Action::SimulateKeyPress {
                key: kc.key,
                modifiers: kc.modifiers
            })).unwrap();

            Ok(state.is_awake())
        }

        fn is_awake() {
            inject state;

            Ok(state.is_awake())
        }

        fn awake() {
            inject state;

            state.tx.send(Event::Action(Action::Awake)).unwrap();

            Ok(())
        }

        fn hibernate() {
            inject state;

            state.tx.send(Event::Action(Action::Hibernate)).unwrap();

            Ok(())
        }

        fn unbind(key: String) {
            inject state;

            state.tx.send(Event::Action(Action::RemoveKeybinding { key }))
                .unwrap();
            Ok(())
        }

        fn update_window_layout() {
            inject state;

            state.tx.send(Event::RenderGraph).unwrap();

            Ok(())
        }

        fn bar_set_layout(layout: BarLayout) {
            inject lua, state;

            lua.set_named_registry_value("left", layout.left)?;
            lua.set_named_registry_value("center", layout.center)?;
            lua.set_named_registry_value("right", layout.right)?;
            state.tx.send(Event::RenderBarLayout).unwrap();

            Ok(())
        }

        fn open_menu() {
            inject state;

            state.tx.send(Event::ShowMenu).unwrap();

            Ok(())
        }

        fn exit() {
            inject state;

            state.tx.send(Event::Exit).unwrap();

            Ok(())
        }

        fn change_ws(ws_id: WorkspaceId) {
            inject state;

            state.tx.send(Event::Action(Action::Workspace(WorkspaceAction::Change(ws_id
            ))))
            .unwrap();

            Ok(())
        }

        fn ws_focus(ws_id: Option<WorkspaceId>, direction: Direction) {
            inject state;

            state.tx.send(Event::Action(Action::Workspace(WorkspaceAction::Focus(
                ws_id, direction,
            ))))
            .unwrap();

            Ok(())
        }

        fn ws_get_focused_win(ws_id: WorkspaceId) {
            inject state;

            Ok(state.with_ws(ws_id, |ws| ws.get_focused_win().map(|w| w.get_id())).flatten())
        }

        fn ws_get_all() {
            inject state;

            let mut workspaces = vec![];

            for d in state.displays.read().iter() {
                let ws_ids = d
                    .wm
                    .workspaces
                    .iter()
                    .map(|w| w.id)
                    .collect::<Vec<_>>();

                for id in ws_ids {
                    workspaces.push(id);
                }
            }

            Ok(workspaces)
        }

        fn ws_swap(ws_id: Option<WorkspaceId>, direction: Direction) {
            inject state;

            state.tx.send(Event::Action(Action::Workspace(WorkspaceAction::Swap(
                ws_id, direction,
            ))))
            .unwrap();

            Ok(())
        }

        fn session_save() {
            inject state;

            state.tx.send(Event::Action(Action::SaveSession)).unwrap();

            Ok(())
        }

        fn session_load() {
            inject state;

            state.tx.send(Event::Action(Action::LoadSession)).unwrap();

            Ok(())
        }

        fn win_close(win_id: Option<WindowId>) {
            inject state;

            state.tx.send(Event::Action(Action::Window(WindowAction::Close(win_id))))
                .unwrap();

            Ok(())
        }

        fn win_is_managed(win_id: Option<WindowId>) {
            inject state;

            let id = win_id.unwrap_or_else(|| Api::get_foreground_window().get_id());
            Ok(state.win_is_managed(id))
        }

        fn win_get_title(win_id: WindowId) {
            inject state;

            Ok(Window::new(win_id).get_title())
        }

        fn win_manage(win_id: Option<WindowId>) {
            inject state;

            state.tx.send(Event::Action(Action::Window(WindowAction::Manage(win_id))))
                .unwrap();

            Ok(())
        }

        fn win_unmanage(win_id: Option<WindowId>) {
            inject state;

            state.tx.send(Event::Action(Action::Window(WindowAction::Unmanage(
                win_id,
            ))))
            .unwrap();

            Ok(())
        }

        fn dsp_get_focused() {
            inject state;

            Ok(state.get_focused_dsp_id())
        }

        fn dsp_get_focused_ws(dsp_id: DisplayId) {
            inject state;

            Ok(state.with_dsp(dsp_id, |dsp| dsp.wm.focused_workspace_id).unwrap())
        }

        fn dsp_contains_ws(dsp_id: Option<DisplayId>, ws_id: WorkspaceId) {
            inject state;

            let dsp_id = dsp_id.unwrap_or_else(|| state.get_focused_dsp_id());

            Ok(state.with_dsp(dsp_id, |d| {
                d.wm.workspaces.iter().any(|ws| ws.id == ws_id)
            }).unwrap_or(false))
        }
    });

    ns.register(None)?;

    // Run the nog init.lua
    rt.eval("dofile(nog.runtime_path .. '/lua/init.lua')")?;

    Ok(rt)
}
