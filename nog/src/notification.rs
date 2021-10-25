use crate::{
    paths::get_bin_path,
    platform::{Area, Position},
};
use rgb::Rgb;
use std::{
    os::windows::process::CommandExt,
    process::{Child, Command},
    time::Instant,
};

const NOTIF_HEIGHT: usize = 60;
const NOTIF_WIDTH: usize = 200;
const NOTIF_PADDING: usize = 20;

#[derive(Debug)]
pub struct NotificationManager {
    notifications: Vec<(Instant, Child)>,
    root_position: Position,
}

impl NotificationManager {
    pub fn new(display_area: &Area) -> Self {
        Self {
            notifications: vec![],
            root_position: Position::new(
                (display_area.size.width - NOTIF_PADDING - NOTIF_WIDTH) as isize,
                NOTIF_PADDING as isize,
            ),
        }
    }

    pub fn push(&mut self, notif: Notification) {
        let idx = self.notifications.len();
        self.notifications.push((
            Instant::now(),
            notif
                .size(NOTIF_WIDTH, NOTIF_HEIGHT)
                .position(
                    self.root_position.x,
                    self.root_position.y + ((NOTIF_PADDING + NOTIF_HEIGHT) * idx) as isize,
                )
                .spawn(),
        ));
    }
}

#[derive(Clone, Debug)]
pub struct Notification {
    width: usize,
    height: usize,
    x: isize,
    y: isize,
    message: String,
    bg: Rgb,
    fg: Rgb,
    font_name: String,
    font_size: usize,
}

impl Notification {
    pub fn new() -> Self {
        Self {
            width: 100,
            height: 100,
            x: 0,
            y: 0,
            message: String::new(),
            bg: Rgb::default(),
            fg: Rgb::default(),
            font_name: String::from("Consolas"),
            font_size: 20,
        }
    }

    pub fn size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn position(mut self, x: isize, y: isize) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn background(mut self, color: Rgb) -> Self {
        self.bg = color;
        self
    }

    pub fn foreground(mut self, color: Rgb) -> Self {
        self.fg = color;
        self
    }

    pub fn font(mut self, name: String, size: usize) -> Self {
        self.font_name = name;
        self.font_size = size;
        self
    }

    pub fn message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn spawn(self) -> Child {
        let mut path = get_bin_path();
        path.push("nog-notif.exe");
        Command::new(path)
            .args(&["-b", &format!("0x{:x}", self.bg.to_hex())])
            .args(&["-t", &format!("0x{:x}", self.fg.to_hex())])
            .args(&["-n", &self.font_name])
            .args(&["-s", &self.font_size.to_string()])
            .args(&["-h", &self.height.to_string()])
            .args(&["-w", &self.width.to_string()])
            .args(&["-x", &self.x.to_string()])
            .args(&["-y", &self.y.to_string()])
            .arg("-m")
            .raw_arg(&format!("\"{}\"", self.message))
            .spawn()
            .unwrap()
    }
}
