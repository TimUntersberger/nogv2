use crate::direction::Direction;
use crate::display::DisplayId;
use crate::key_combination::KeyCombination;
use crate::keybinding::KeybindingMode;
use crate::platform::{MonitorId, Size, WindowId};
use crate::workspace::WorkspaceId;
use mlua::prelude::*;
use rgb::Rgb;
use std::str::FromStr;

use super::LuaEvent;

impl<'lua> FromLua<'lua> for KeybindingMode {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        if let Ok(string) = String::from_lua(lua_value.clone(), lua) {
            if let Ok(mode) = KeybindingMode::from_str(&string) {
                return Ok(mode);
            }
        }

        Err(LuaError::FromLuaConversionError {
            from: lua_value.type_name(),
            to: "KeybindingMode",
            message: Some("Expected one of the following strings: 'g', 'w', 'n'".into()),
        })
    }
}

impl<'lua> FromLua<'lua> for KeyCombination {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match String::from_lua(lua_value.clone(), lua) {
            Ok(string) => match KeyCombination::from_str(&string) {
                Ok(kc) => Ok(kc),
                Err(msg) => Err(LuaError::FromLuaConversionError {
                    from: lua_value.type_name(),
                    to: "KeyCombination",
                    message: Some(msg),
                }),
            },
            Err(_) => Err(LuaError::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "KeyCombination",
                message: Some("Expected a type that can be coerced into a string".into()),
            }),
        }
    }
}

impl<'lua> ToLua<'lua> for WindowId {
    fn to_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(mlua::Value::Number(self.0 as f64))
    }
}

impl<'lua> FromLua<'lua> for WindowId {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        Ok(WindowId(usize::from_lua(lua_value, lua)?))
    }
}

impl<'lua> ToLua<'lua> for DisplayId {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let s = lua.create_string(&self.0)?;

        s.to_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for DisplayId {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match String::from_lua(lua_value.clone(), lua) {
            Ok(string) => Ok(DisplayId(string)),
            Err(_) => Err(LuaError::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "DisplayId",
                message: Some("Expected a type that can be coerced into a string".into()),
            }),
        }
    }
}

impl<'lua> ToLua<'lua> for WorkspaceId {
    fn to_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(mlua::Value::Number(self.0 as f64))
    }
}

impl<'lua> FromLua<'lua> for WorkspaceId {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        Ok(WorkspaceId(usize::from_lua(lua_value, lua)?))
    }
}

impl<'lua> ToLua<'lua> for MonitorId {
    fn to_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(mlua::Value::Number(self.0 as f64))
    }
}

impl<'lua> FromLua<'lua> for MonitorId {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        Ok(MonitorId(isize::from_lua(lua_value, lua)?))
    }
}

impl<'lua> ToLua<'lua> for Direction {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let s = lua.create_string(&self.to_string())?;

        s.to_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for Direction {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match String::from_lua(lua_value.clone(), lua) {
            Ok(string) => match Direction::from_str(&string) {
                Ok(x) => Ok(x),
                Err(msg) => Err(LuaError::FromLuaConversionError {
                    from: lua_value.type_name(),
                    to: "Direction",
                    message: Some(msg),
                }),
            },
            Err(_) => Err(LuaError::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "Direction",
                message: Some("Expected a type that can be coerced into a string".into()),
            }),
        }
    }
}

impl<'lua> ToLua<'lua> for Size {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let tbl = lua.create_table()?;

        tbl.set("width", self.width)?;
        tbl.set("height", self.height)?;

        Ok(mlua::Value::Table(tbl))
    }
}

impl<'lua> ToLua<'lua> for LuaEvent {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(match self {
            LuaEvent::Manage { manual, win_id } => {
                let tbl = lua.create_table()?;
                tbl.raw_set("manual", manual)?;
                tbl.raw_set("win_id", win_id)?;
                mlua::Value::Table(tbl)
            }
        })
    }
}
