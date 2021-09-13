use std::{
    io,
    process::{Child, Command},
};

use crate::{paths::get_bin_path, platform::MonitorId};

fn create_command() -> Command {
    let mut path = get_bin_path();
    path.push("nog-bar.exe");

    Command::new(path)
}

pub struct Bar {
    process: Child,
    display_id: MonitorId,
}

impl Bar {
    pub fn new(display_id: MonitorId) -> io::Result<Self> {
        Ok(Self {
            process: create_command().spawn()?,
            display_id,
        })
    }
}
