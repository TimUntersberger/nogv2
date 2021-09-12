use std::sync::mpsc::Sender;

use crate::{action::{Action, UpdateConfigActionFn}, config::Config, event::Event, rgb::RGB};
use mlua::prelude::*;

pub struct ConfigProxy {
    config: Config,
    tx: Sender<Event>,
}

impl ConfigProxy {
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            config: Config::default(),
            tx,
        }
    }
}

impl mlua::UserData for ConfigProxy {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, key: String| {
            macro_rules! get_by_string {
                {$($name:ident),*} => {
                    match key.as_str() {
                        $(stringify!($name) => {
                            Some(this.config.$name.clone().to_lua(lua)?)
                        },)*
                        _ => None,
                    }
                }
            }

            let value = get_by_string! {
                color,
                bar_height,
                font_size,
                font_name,
                use_border,
                enable_hot_reloading,
                min_width,
                min_height,
                work_mode,
                light_theme,
                multi_monitor,
                launch_on_startup,
                outer_gap,
                inner_gap,
                remove_decorations,
                remove_task_bar,
                ignore_fullscreen_actions,
                display_app_bar
            };

            Ok(value)
        });

        methods.add_meta_method_mut(
            LuaMetaMethod::NewIndex,
            |lua, this, (key, value): (String, mlua::Value)| {
                macro_rules! update_action_creator {
                    {$($name:ident : $ty:ty),*} => {
                        match key.as_str() {
                            $(stringify!($name) => {
                                let value = <$ty>::from_lua(value, lua)?;
                                Some(UpdateConfigActionFn::new(move |config: &mut Config| {
                                    config.$name = value.clone();
                                }))
                            }),*
                            _ => None,
                        }
                    }
                }

                let maybe_action = update_action_creator! {
                    color: RGB,
                    bar_height: u32,
                    font_size: u32,
                    font_name: String,
                    use_border: bool,
                    enable_hot_reloading: bool,
                    min_width: usize,
                    min_height: usize,
                    work_mode: bool,
                    light_theme: bool,
                    multi_monitor: bool,
                    launch_on_startup: bool,
                    outer_gap: u32,
                    inner_gap: u32,
                    remove_decorations: bool,
                    remove_task_bar: bool,
                    ignore_fullscreen_actions: bool,
                    display_app_bar: bool
                };

                if let Some(f) = maybe_action {
                    f.0(&mut this.config);
                    this.tx
                        .send(Event::Action(Action::UpdateConfig { key, update_fn: f }))
                        .unwrap();
                }

                Ok(())
            },
        );
    }

    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}
}
