use crate::{
    paths::get_bin_path,
    platform::{Area, Position},
    thread_safe::ThreadSafe,
};
use rgb::Rgb;
use std::{
    mem,
    os::windows::process::CommandExt,
    process::{Child, Command},
    sync::mpsc::{sync_channel, SyncSender},
    thread::{self, JoinHandle},
    time::Instant,
};

const NOTIF_HEIGHT: usize = 60;
const NOTIF_WIDTH: usize = 200;
const NOTIF_PADDING: usize = 20;
const NOTIF_TTL: usize = 3500;

#[derive(Debug)]
enum NotificationManagerMessage {
    Exit,
    NotificationClosed(usize),
    Reorganize,
}

#[derive(Debug)]
pub struct NotificationManager {
    /// Used for automatic id generation. Always holds the biggest id.
    cur_id: usize,
    notifications: ThreadSafe<Vec<(usize, Notification, JoinHandle<()>)>>,
    tx: SyncSender<NotificationManagerMessage>,
    root_position: Position,
}

impl NotificationManager {
    pub fn new(display_area: &Area) -> Self {
        let (tx, rx) = sync_channel(10);
        let notifications: ThreadSafe<Vec<(usize, Notification, JoinHandle<()>)>> = ThreadSafe::default();

        {
            let tx = tx.clone();
            let notifications = notifications.clone();
            thread::spawn(move || {
                for msg in rx {
                    match msg {
                        NotificationManagerMessage::Exit => break,
                        NotificationManagerMessage::NotificationClosed(id) => {
                            let new_value = mem::take(&mut *dbg!(notifications.write()))
                                .into_iter()
                                .filter(|x| (*x).0 != id)
                                .collect();

                            *notifications.write() = dbg!(new_value);

                            tx.send(NotificationManagerMessage::Reorganize).unwrap();
                        }
                        NotificationManagerMessage::Reorganize => {
                            //TODO: don't know how to do this yet
                            //
                            //Maybe making nog-notif optionally interactive via a flag so we can
                            //move/resize the notification using stdin
                        }
                    }
                }
            });
        }

        Self {
            notifications,
            tx,
            cur_id: 0,
            root_position: Position::new(
                (display_area.size.width - NOTIF_PADDING - NOTIF_WIDTH) as isize,
                NOTIF_PADDING as isize,
            ),
        }
    }

    fn gen_id(&mut self) -> usize {
        self.cur_id += 1;

        self.cur_id - 1
    }

    fn position_notif(&self, notif: Notification, idx: usize) -> Notification {
        notif.size(NOTIF_WIDTH, NOTIF_HEIGHT).position(
            self.root_position.x,
            self.root_position.y + ((NOTIF_PADDING + NOTIF_HEIGHT) * idx) as isize,
        )
    }

    pub fn push(&mut self, notif: Notification) {
        let id = dbg!(self.gen_id());
        let idx = dbg!(self.notifications.read()).len();

        let tx = self.tx.clone();
        let notif_clone = notif.clone();

        let notif = self.position_notif(notif, idx);
        
        let handle = thread::spawn(move || {
            let mut child_process = notif.spawn();

            child_process.wait().unwrap();

            let _ = tx.send(NotificationManagerMessage::NotificationClosed(id));
        });

        self.notifications.write().push((id, notif_clone, handle));
    }
}

impl Drop for NotificationManager {
    fn drop(&mut self) {
        let _ = self.tx.send(NotificationManagerMessage::Exit);
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

    pub fn spawn(&self) -> Child {
        let mut path = get_bin_path();
        path.push("nog-notif.exe");
        // If the unwrap panics with: The system cannot find the file specified.
        //
        // You might have to run the following command:
        //
        // ```
        // cargo build -p nog-notif
        // ```
        Command::new(path)
            .args(&["-b", &format!("0x{:x}", self.bg.to_hex())])
            .args(&["-t", &format!("0x{:x}", self.fg.to_hex())])
            .args(&["-n", &self.font_name])
            .args(&["-s", &self.font_size.to_string()])
            .args(&["-h", &self.height.to_string()])
            .args(&["-w", &self.width.to_string()])
            .args(&["-x", &self.x.to_string()])
            .args(&["-y", &self.y.to_string()])
            .args(&["--ttl", &NOTIF_TTL.to_string()])
            .arg("-m")
            .raw_arg(&format!("\"{}\"", self.message))
            .spawn()
            .unwrap()
    }
}
