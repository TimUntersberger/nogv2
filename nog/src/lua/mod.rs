pub mod config_proxy;
pub mod conversions;
pub mod graph_proxy;
pub mod namespace;
pub mod runtime;

use std::{os::windows::process::CommandExt, process::Command};

pub use namespace::LuaNamespace;
pub use runtime::LuaRuntime;

use mlua::prelude::*;

use crate::{
    action::{Action, WindowAction, WorkspaceAction},
    constants::get_version,
    direction::Direction,
    display::DisplayId,
    event::Event,
    file_watcher::FileWatcher,
    key_combination::KeyCombination,
    keybinding::KeybindingMode,
    lua::config_proxy::ConfigProxy,
    notification::Notification,
    paths::{get_config_path, get_runtime_path},
    platform::{Api, NativeApi, NativeWindow, Window, WindowId},
    state::State,
    workspace::{WorkspaceId, WorkspaceState},
};
use rgb::Rgb;

pub fn lua_error_to_string(err: LuaError) -> String {
    match err {
        LuaError::CallbackError { traceback, cause } => {
            format!("{}\n{}", cause, traceback)
        }
        _ => format!("{}", err),
    }
}

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

/// The events that can be listened to using `nog.on`
#[derive(Clone, Debug)]
pub enum LuaEvent {
    WinManage {
        /// whether the user tries to manage the window via nog.win_manage
        manual: bool,
        ws_id: Option<WorkspaceId>,
        win_id: WindowId,
    },
    WsCreated {
        ws_id: WorkspaceId,
    },
}

pub fn init_events(rt: &LuaRuntime) -> LuaResult<()> {
    rt.lua
        .set_named_registry_value("win_manage", rt.lua.create_table()?)?;

    rt.lua
        .set_named_registry_value("ws_created", rt.lua.create_table()?)?;

    Ok(())
}

pub fn get_event_handlers_iter<'a>(
    rt: &'a LuaRuntime,
    event_name: &str,
) -> LuaResult<impl Iterator<Item = mlua::Function<'a>>> {
    Ok(rt
        .lua
        .named_registry_value::<str, mlua::Table>(event_name)
        .map_err(|_| mlua::Error::RuntimeError(format!("Event '{}' doesn't exist", event_name)))?
        .raw_sequence_values::<mlua::Function>()
        .flatten())
}

/// Returns whether any event handler returned false
pub fn emit_win_manage(rt: &LuaRuntime, event: LuaEvent) -> LuaResult<()> {
    for ev_handler in get_event_handlers_iter(rt, "win_manage")? {
        ev_handler.call::<LuaEvent, Option<mlua::Table>>(event.clone())?;
    }

    Ok(())
}

/// Returns whether any event handler returned false
pub fn emit_ws_created(rt: &LuaRuntime, event: LuaEvent) -> LuaResult<()> {
    for ev_handler in get_event_handlers_iter(rt, "ws_created")? {
        ev_handler.call::<LuaEvent, Option<mlua::Table>>(event.clone())?;
    }

    Ok(())
}

pub fn init(state: State) -> LuaResult<LuaRuntime> {
    let rt = LuaRuntime::new(state.clone())?;

    let ns = namespace!(rt, nog, {
            const runtime_path = get_runtime_path().to_str().unwrap();
            const config_path = get_config_path().to_str().unwrap();
            const version = get_version();
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

            fn notify(settings: mlua::Table) {
                inject state, lua;

                let mut background = state.config.read().color;
                let mut foreground = state.config.read().get_text_color();
                let mut message = None;

                for (key, val) in settings.pairs::<String, mlua::Value>().flatten() {
                    match key.as_str() {
                        "background" => match Rgb::from_lua(val, &lua) {
                            Ok(x) => background = x,
                            Err(_) => {},
                        },
                        "foreground" => match Rgb::from_lua(val, &lua) {
                            Ok(x) => foreground = x,
                            Err(_) => {},
                        },
                        "message" => message = Some(String::from_lua(val, &lua)?),
                        _ => {}
                    }
                }

                if message.is_none() {
                    return Err(mlua::Error::RuntimeError(String::from("nog.notify requires a `message` property")));
                }

                state.tx.send(Event::Action(Action::CreateNotification(
                    Notification::new()
                        .background(background)
                        .foreground(foreground)
                        .message(message.unwrap()),
                )))
                .unwrap();
                Ok(())
            }

            fn on(event_name: String, cb: mlua::Function) {
                inject lua;

                let tbl: mlua::Table = lua
                    .named_registry_value(&event_name)
                    .map_err(|_| mlua::Error::RuntimeError(format!("Event '{}' doesn't exist", &event_name)))?;
                let len = tbl.len()?;

                tbl.raw_insert(len + 1, cb)?;

                Ok(())
            }

            fn watch(path: String, cb: mlua::Function<'static>) {
                inject lua, state;

                let key = lua.create_registry_value(cb)?;

                let mut fw = FileWatcher::new(path.into(), state.tx.clone(), key);

                fw.start();

                Ok(fw)
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

            fn move_win_to_ws(win_id: Option<WindowId>, ws_id: WorkspaceId) {
                inject state;

                state.tx.send(Event::Action(Action::MoveWindowToWorkspace(win_id, ws_id)))
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

            fn ws_is_fullscreen(ws_id: WorkspaceId) {
                inject state;

                Ok(state.with_ws(ws_id, |ws| ws.is_fullscreen()))
            }

            fn ws_set_fullscreen(ws_id: Option<WorkspaceId>, value: bool) {
                inject state;

                state.tx.send(Event::Action(Action::Workspace(WorkspaceAction::SetFullscreen(ws_id, value)))).unwrap();

                Ok(())
            }

            fn ws_set_name(ws_id: Option<WorkspaceId>, value: String) {
                inject state;

                state.tx.send(Event::Action(Action::Workspace(WorkspaceAction::SetName(ws_id, value)))).unwrap();

                Ok(())
            }

            fn ws_get_name(ws_id: Option<WorkspaceId>) {
                inject state;

                Ok(
                    state.with_ws(
                        ws_id.unwrap_or_else(|| state.get_focused_ws_id().unwrap()),
                        |ws|ws.display_name.clone()
                    )
                )
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

            fn session_save(name: String) {
                inject state;

                state.tx.send(Event::Action(Action::SaveSession(name))).unwrap();

                Ok(())
            }

            fn session_load(name: String) {
                inject state;

                state.tx.send(Event::Action(Action::LoadSession(name))).unwrap();

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

            fn win_minimize(win_id: Option<WindowId>) {
                inject state;

                state.tx.send(Event::Action(Action::Window(WindowAction::Minimize(win_id))))
                    .unwrap();

                Ok(())
            }

            fn win_get_title(win_id: WindowId) {
                inject state;

                Ok(Window::new(win_id).get_title())
            }

            fn win_get_size(win_id: WindowId) {
                inject state;

                Ok(Window::new(win_id).get_size())
            }

            fn launch(path: String) {
                inject state;

                state.tx.send(Event::Action(Action::Launch(path))).unwrap();

                Ok(())
            }

            fn win_manage(win_id: Option<WindowId>) {
                inject state;

                state.tx.send(Event::Action(Action::Window(WindowAction::Manage(None, win_id))))
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

            fn dsp_get_wss(dsp_id: Option<DisplayId>) {
                inject state;

                let dsp_id = dsp_id.unwrap_or_else(|| state.get_focused_dsp_id());

                Ok(state.with_dsp(dsp_id, |dsp| dsp.wm.workspaces.iter().map(|ws| ws.id).collect::<Vec<_>>()))
            }

            fn dsp_get_focused() {
                inject state;

                Ok(state.get_focused_dsp_id())
            }

            fn dsp_get_focused_ws(dsp_id: Option<DisplayId>) {
                inject state;

                let dsp_id = dsp_id.unwrap_or_else(|| state.get_focused_dsp_id());

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

    init_events(&rt)?;

    // Run the nog init.lua
    rt.eval("dofile(nog.runtime_path .. '/lua/init.lua')")?;

    Ok(rt)
}
