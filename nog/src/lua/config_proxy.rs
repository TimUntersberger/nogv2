use crate::{
    action::{Action, UpdateConfigActionFn},
    config::{Config, ConfigProperty},
    event::Event,
    thread_safe::ThreadSafe,
};
use mlua::prelude::*;
use rgb::Rgb;
use std::{mem, sync::mpsc::SyncSender};

pub struct ConfigProxy {
    config: ThreadSafe<Config>,
    tx: SyncSender<Event>,
}

impl ConfigProxy {
    pub fn new(tx: SyncSender<Event>, config: ThreadSafe<Config>) -> Self {
        Self { config, tx }
    }
}

impl mlua::UserData for ConfigProxy {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, key: String| {
            macro_rules! get_by_string {
                {$($name:ident),*} => {
                    match key.as_str() {
                        $(stringify!($name) => {
                            Some(this.config.read().$name.clone().to_lua(lua)?)
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
                light_theme,
                multi_monitor,
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
                macro_rules! change_detector {
                    {$($name:ident : $ty:ty => $enum:ident),*} => {
                        match key.as_str() {
                            $(stringify!($name) => {
                                let value = <$ty>::from_lua(value, lua)?;
                                let old_value = mem::replace(&mut this.config.write().$name, value);
                                Some(ConfigProperty::$enum(old_value))
                            }),*
                            _ => None,
                        }
                    }
                }

                let config_prop = change_detector! {
                    color: Rgb => Color,
                    bar_height: u32 => BarHeight,
                    font_size: u32 => FontSize,
                    font_name: String => FontName,
                    light_theme: bool => LightTheme,
                    multi_monitor: bool => MultiMonitor,
                    outer_gap: u32 => OuterGap,
                    inner_gap: u32 => InnerGap,
                    remove_decorations: bool => RemoveDecorations,
                    remove_task_bar: bool => RemoveTaskBar,
                    ignore_fullscreen_actions: bool => IgnoreFullscreenActions,
                    display_app_bar: bool => DisplayAppBar
                };

                if let Some(prop) = config_prop {
                    this.tx
                        .send(Event::Action(Action::UpdateConfig(prop)))
                        .unwrap();
                }

                Ok(())
            },
        );
    }

    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}
}
