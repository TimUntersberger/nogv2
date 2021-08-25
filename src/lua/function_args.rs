pub trait FromLuaFunctionArgs<'a>: Sized {
    fn from_args(args: &'a LuaFunctionArgs) -> LuaResult<Self>;
}

impl<'a, T> FromLuaFunctionArgs<'a> for T where T: FromLua<'a> + Clone {
    fn from_args(args: &'a LuaFunctionArgs) -> LuaResult<Self> {
        Ok(T::from_lua(args.0[0].clone(), args.1)?)
    }
}

#[derive(Clone)]
pub struct LuaFunctionArgs<'a>(Vec<mlua::Value<'a>>, &'a Lua);

impl<'a> FromLuaMulti<'a> for LuaFunctionArgs<'a> {
    fn from_lua_multi(v: LuaMultiValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
        Ok(Self(v.into_vec(), lua))
    }
}

impl<'a> LuaFunctionArgs<'a> {
    pub fn validate<T>(&'a self) -> LuaResult<T>
    where
        T: FromLuaFunctionArgs<'a>,
    {
        T::from_args(&self)
    }
}
