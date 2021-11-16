use crate::{
    paths::get_bin_path,
    platform::{Area, NativeWindow, Position, Window, WindowId},
    thread_safe::ThreadSafe,
};
use rgb::Rgb;
use std::{
    io::{BufRead, BufReader, Read},
    mem,
    os::windows::process::CommandExt,
    process::{Child, Command, Stdio},
    sync::mpsc::{sync_channel, SyncSender},
    thread::{self, JoinHandle},
    time::Instant,
};

const NOTIF_HEIGHT: usize = 60;
const NOTIF_WIDTH: usize = 200;
const NOTIF_PADDING: usize = 20;
const NOTIF_TTL: usize = 3500;

#[derive(Debug)]
struct ManagedNotification {
    id: usize,
    notif: Notification,
    process: JoinHandle<()>,
    window: Window,
}

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
    notifications: ThreadSafe<Vec<ManagedNotification>>,
    tx: SyncSender<NotificationManagerMessage>,
    root_position: Position,
}

impl NotificationManager {
    pub fn new(display_area: &Area) -> Self {
        let (tx, rx) = sync_channel(10);
        let notifications: ThreadSafe<Vec<ManagedNotification>> = ThreadSafe::default();
        let root_position = Position::new(
            (display_area.size.width - NOTIF_PADDING - NOTIF_WIDTH) as isize,
            NOTIF_PADDING as isize,
        );

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
                                .filter(|x| (*x).id != id)
                                .collect();

                            *notifications.write() = dbg!(new_value);

                            tx.send(NotificationManagerMessage::Reorganize).unwrap();
                        }
                        NotificationManagerMessage::Reorganize => {
                            for (idx, managed_notif) in notifications.read().iter().enumerate() {
                                let pos = Position::new(
                                    root_position.x,
                                    root_position.y
                                        + ((NOTIF_PADDING + NOTIF_HEIGHT) * idx) as isize,
                                );
                                managed_notif.window.reposition(pos);
                            }
                        }
                    }
                }
            });
        }

        Self {
            notifications,
            tx,
            cur_id: 0,
            root_position,
        }
    }

    fn gen_id(&mut self) -> usize {
        self.cur_id += 1;

        self.cur_id - 1
    }

    fn calculate_notif_position(&self, idx: usize) -> (isize, isize) {
        (
            self.root_position.x,
            self.root_position.y + ((NOTIF_PADDING + NOTIF_HEIGHT) * idx) as isize,
        )
    }

    pub fn push(&mut self, notif: Notification) {
        let id = dbg!(self.gen_id());
        let idx = dbg!(self.notifications.read()).len();

        let tx = self.tx.clone();

        let notif_pos = self.calculate_notif_position(idx);
        let notif = notif
            .size(NOTIF_WIDTH, NOTIF_HEIGHT)
            .position(notif_pos.0, notif_pos.1);
        let notif_clone = notif.clone();

        let (win_tx, win_rx) = sync_channel(1);

        let handle = thread::spawn(move || {
            let mut child_process = notif.spawn();

            if let Some(mut stdout) = child_process.stdout.take() {
                let mut out = Vec::new();

                BufReader::new(stdout)
                    .read_until('\n' as u8, &mut out)
                    .expect("Failed to read notification window id");

                let id_len = out.len() - 1;
                let window_id = String::from_utf8(out.into_iter().take(id_len).collect())
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();

                win_tx.send(window_id).unwrap();
            }

            child_process.wait().unwrap();

            let _ = tx.send(NotificationManagerMessage::NotificationClosed(id));
        });

        let win_id = win_rx.recv().unwrap();

        self.notifications.write().push(ManagedNotification {
            id,
            notif: notif_clone,
            process: handle,
            window: Window::new(WindowId(win_id)),
        });
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
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("-v")
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
