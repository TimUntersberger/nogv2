use crate::keybinding::KeybindingMode;
use crate::key_combination::KeyCombination;
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
