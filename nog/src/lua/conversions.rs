use crate::direction::Direction;
use crate::key_combination::KeyCombination;
use crate::keybinding::KeybindingMode;
use crate::platform::WindowId;
use crate::rgb::RGB;
use crate::workspace::WorkspaceId;
use crate::display::DisplayId;
use mlua::prelude::*;
use std::str::FromStr;

impl<'lua> FromLua<'lua> for KeybindingMode {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        if let Ok(string) = String::from_lua(lua_value.clone(), lua) {
            if let Ok(mode) = KeybindingMode::from_str(&string) {
                return Ok(mode);
            }
        }

        Err(LuaError::FromLuaConversionError {
            from: lua_value.type_name(),
            to: "KeybindingMode".into(),
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
                    to: "KeyCombination".into(),
                    message: Some(msg),
                }),
            },
            Err(_) => Err(LuaError::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "KeyCombination".into(),
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

impl<'lua> ToLua<'lua> for DisplayId {
    fn to_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(mlua::Value::Number(self.0 as f64))
    }
}

impl<'lua> FromLua<'lua> for DisplayId {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        Ok(DisplayId(usize::from_lua(lua_value, lua)?))
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
                    to: "Direction".into(),
                    message: Some(msg),
                }),
            },
            Err(_) => Err(LuaError::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "Direction".into(),
                message: Some("Expected a type that can be coerced into a string".into()),
            }),
        }
    }
}

impl<'lua> ToLua<'lua> for RGB {
    fn to_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(LuaValue::Number(self.to_hex() as f64))
    }
}

impl<'lua> FromLua<'lua> for RGB {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match i32::from_lua(lua_value.clone(), lua) {
            Ok(x) => Ok(RGB::from_hex(x)),
            Err(_) => Err(LuaError::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "RGB".into(),
                message: Some("Expected a type that can be coerced into a string".into()),
            }),
        }
    }
}
