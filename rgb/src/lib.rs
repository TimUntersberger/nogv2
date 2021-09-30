#[derive(Default, Clone, Copy, Debug)]
pub struct Rgb(pub [f32; 3]);

impl Rgb {
    pub const WHITE: Rgb = Rgb([1.0, 1.0, 1.0]);
    pub const BLACK: Rgb = Rgb([0.0, 0.0, 0.0]);

    pub fn from_hex(hex: i32) -> Self {
        Rgb([
            ((hex >> 16) & 0xFF) as f32 / 255.0,
            ((hex >> 8) & 0xFF) as f32 / 255.0,
            (hex & 0xFF) as f32 / 255.0,
        ])
    }

    pub fn to_hex(self) -> i32 {
        (((self.0[0] * 255.0) as i32 & 0xff) << 16)
            + (((self.0[1] * 255.0) as i32 & 0xff) << 8)
            + ((self.0[2] * 255.0) as i32 & 0xff)
    }

    pub fn scaled(&self, factor: f32) -> Self {
        let [mut red, mut green, mut blue] = self.0;

        red *= factor;
        green *= factor;
        blue *= factor;

        Self([red.min(1.0), green.min(1.0), blue.min(1.0)])
    }
}

#[cfg(feature = "lua")]
use mlua::prelude::*;

#[cfg(feature = "lua")]
impl<'lua> ToLua<'lua> for Rgb {
    fn to_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(LuaValue::Number(self.to_hex() as f64))
    }
}

#[cfg(feature = "lua")]
impl<'lua> FromLua<'lua> for Rgb {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match i32::from_lua(lua_value.clone(), lua) {
            Ok(x) => Ok(Rgb::from_hex(x)),
            Err(_) => Err(LuaError::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "Rgb",
                message: Some("Expected a type that can be coerced into a string".into()),
            }),
        }
    }
}
