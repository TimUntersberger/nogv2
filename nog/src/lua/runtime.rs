use std::{collections::HashMap, sync::{Arc, RwLock, mpsc::Sender}};

use crate::{event::Event, window_manager::WindowManager};

use super::LuaNamespace;
use mlua::prelude::*;

pub struct LuaRuntime<'a> {
    pub rt: &'static Lua,
    pub namespace: LuaNamespace<'a>,
    tx: Sender<Event>,
    wm: Arc<RwLock<WindowManager>>
}

impl<'a> LuaRuntime<'a> {
    pub fn new(tx: Sender<Event>, wm: Arc<RwLock<WindowManager>>) -> LuaResult<Self> {
        let options = LuaOptions::default();
        let rt = unsafe { Lua::unsafe_new_with(mlua::StdLib::ALL, options) }.into_static();
        Ok(Self {
            rt,
            namespace: LuaNamespace::new(&rt, tx.clone(), wm.clone(), "nog")?,
            tx,
            wm
        })
    }

    pub fn eval(&self, s: &str) -> LuaResult<mlua::Value> {
        self.rt.load(s).eval()
    }

    pub fn create_namespace(&self, s: &str) -> LuaResult<LuaNamespace<'a>> {
        LuaNamespace::new(self.rt, self.tx.clone(), self.wm.clone(), s)
    }

    pub fn call_fn<A>(&'a self, path: &str, args: A) -> LuaResult<mlua::Value>
    where
        A: ToLuaMulti<'a>,
    {
        mlua::Function::from_lua(self.eval(path)?, self.rt)?.call(args)
    }
}
