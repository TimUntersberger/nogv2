use std::{
    io,
    process::{Child, Command},
};

use crate::paths::get_bin_path;

fn create_command() -> Command {
    let mut path = get_bin_path();
    path.push("nog-bar.exe");

    Command::new(path)
}

#[derive(Debug)]
pub struct Bar {
    process: Child,
}

impl Bar {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            process: create_command().spawn()?,
        })
    }

    pub fn close(&mut self) {
        self.process.kill();
    }
}
