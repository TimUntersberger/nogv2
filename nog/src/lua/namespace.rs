use mlua::prelude::*;

pub struct LuaNamespace<'a> {
    rt: &'a Lua,
    name: String,
    tbl: mlua::Table<'a>,
    namespaces: Vec<LuaNamespace<'a>>,
}

impl<'a> LuaNamespace<'a> {
    pub fn new(lua: &'a Lua, name: &str) -> LuaResult<Self> {
        Ok(Self {
            rt: lua,
            name: name.to_string(),
            tbl: lua.create_table()?,
            namespaces: vec![],
        })
    }

    pub fn add_function<F, A, FReturn>(&self, name: &str, f: F) -> LuaResult<()>
    where
        F: Fn(A) -> LuaResult<FReturn> + Send + 'static,
        A: FromLuaMulti<'a>,
    {
        self.tbl.set(
            name,
            self.rt.create_function(move |_lua, args: A| {
                f(args);
                Ok(())
            })?,
        )
    }

    pub fn add_constant<T>(&self, name: &str, value: T) -> LuaResult<()>
    where
        T: ToLua<'a>,
    {
        self.tbl.set(name, value)
    }

    pub fn add_namespace(&mut self, ns: LuaNamespace<'a>) {
        self.namespaces.push(ns);
    }

    pub fn register(&self, parent: Option<&LuaNamespace<'a>>) -> LuaResult<()> {
        for namespace in &self.namespaces {
            namespace.register(Some(&self))?;
        }

        match parent {
            Some(parent) => parent.tbl.set(self.name.clone(), self.tbl.clone()),
            None => self.rt.globals().set(self.name.clone(), self.tbl.clone()),
        }
    }
}
