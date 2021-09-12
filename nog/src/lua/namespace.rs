use std::sync::{mpsc::Sender, Arc, RwLock};

use mlua::prelude::*;

use crate::{
    event::Event, state::State, types::ThreadSafeWindowManagers, window_manager::WindowManager,
};

pub struct LuaNamespace {
    pub lua: &'static Lua,
    state: State,
    name: String,
    tbl: mlua::Table<'static>,
    namespaces: Vec<LuaNamespace>,
}

impl LuaNamespace {
    pub fn new(state: State, lua: &'static Lua, name: &str) -> LuaResult<Self> {
        Ok(Self {
            state,
            lua,
            name: name.to_string(),
            tbl: lua.create_table()?,
            namespaces: vec![],
        })
    }

    pub fn add_function<'a, F, A, FReturn>(&self, name: &str, f: F) -> LuaResult<()>
    where
        FReturn: ToLuaMulti<'a>,
        F: Fn(&State, &mlua::Lua, A) -> LuaResult<FReturn> + 'static,
        A: FromLuaMulti<'a>,
    {
        let state = self.state.clone();
        let lua = self.lua;
        self.tbl.set(
            name,
            self.lua
                .create_function(move |lua, args: A| f(&state, lua, args))?,
        )
    }

    pub fn add_constant<T>(&self, name: &str, value: T) -> LuaResult<()>
    where
        T: ToLua<'static>,
    {
        self.tbl.set(name, value)
    }

    pub fn add_namespace(&mut self, ns: LuaNamespace) {
        self.namespaces.push(ns);
    }

    pub fn register(&self, parent: Option<&LuaNamespace>) -> LuaResult<()> {
        for namespace in &self.namespaces {
            namespace.register(Some(&self))?;
        }

        match parent {
            Some(parent) => parent.tbl.set(self.name.clone(), self.tbl.clone()),
            None => self.lua.globals().set(self.name.clone(), self.tbl.clone()),
        }
    }
}
