use mlua::prelude::*;
use crate::keybinding::KeybindingMode;
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
            message: Some("Expected one of the following strings: 'g', 'w', 'n'".into())
        })
    }
}
