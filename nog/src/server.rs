use crate::action::ExecuteLuaActionFn;
use crate::display::Display;
use crate::platform::NativeWindow;
use crate::state::State;
use crate::{action::Action, event::Event, thread_safe::ThreadSafe};
use nog_protocol::{BarContent, Message};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{sync_channel, SyncSender},
};

pub struct Server {
    tx: SyncSender<Event>,
    state: State,
    host: String,
    port: u32,
}

impl Server {
    pub fn new(tx: SyncSender<Event>, state: State) -> Self {
        Self {
            tx,
            state,
            host: "localhost".into(),
            port: 8080,
        }
    }
    pub fn spawn(tx: SyncSender<Event>, state: State) {
        std::thread::spawn(move || {
            let server = Server::new(tx, state);
            server.start();
        });
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).unwrap();

        for stream in listener.incoming().flatten() {
            let tx = self.tx.clone();
            let state = self.state.clone();
            std::thread::spawn(move || {
                if let Err(_e) = handle_client(stream, tx, state) {
                    log::error!("{:?}", _e);
                }
            });
        }
    }
}

fn handle_client(
    mut stream: TcpStream,
    tx: SyncSender<Event>,
    state: State,
) -> std::io::Result<()> {
    loop {
        let mut header_buffer = [0u8; 2];
        stream.read_exact(&mut header_buffer)?;

        let msg_len = u16::from_be_bytes(header_buffer);

        let mut msg_buf = vec![0u8; msg_len as usize];
        stream.read_exact(&mut msg_buf)?;

        let msg = String::from_utf8(msg_buf).unwrap();

        if let Ok(msg) = Message::deserialize(&msg) {
            let response = match msg {
                Message::GetBarContent => serde_json::to_string(&*state.bar_content.read())
                    .expect("Serde failed to serialize the bar content"),
                Message::GetState => {
                    let mut pstate = nog_protocol::State::default();
                    pstate.focused_display_id = state.get_focused_dsp_id().0.clone();
                    pstate.displays = state
                        .displays
                        .read()
                        .iter()
                        .map(|d| nog_protocol::Display {
                            id: d.id.0.clone(),
                            monitor_id: d.monitor.id.0 as usize,
                            focused_workspace_id: d.wm.focused_workspace_id.map(|x| x.0),
                            workspaces: d
                                .wm
                                .workspaces
                                .iter()
                                .map(|ws| nog_protocol::Workspace {
                                    id: ws.id.0,
                                    layout: ws.layout_name.clone(),
                                    focused_window_id: ws
                                        .get_focused_win()
                                        .map(|win| win.get_id().0),
                                    windows: ws
                                        .windows()
                                        .map(|id| nog_protocol::Window { id: id.0 })
                                        .collect(),
                                })
                                .collect(),
                        })
                        .collect();
                    serde_json::to_string(&pstate).expect("Serde failed to serialize the state")
                }
                Message::ExecuteLua { code, print_type } => {
                    let (result_tx, result_rx) = sync_channel(1);

                    tx.send(Event::Action(Action::ExecuteLua {
                        code,
                        print_type,
                        capture_stdout: true,
                        cb: ExecuteLuaActionFn::new(move |res| {
                            result_tx.send(res).unwrap();
                        }),
                    }))
                    .unwrap();

                    let res = result_rx.recv();

                    match res.unwrap() {
                        Ok(output) => format!("Ok:{}", output),
                        // TODO: add support for incomplete Syntax
                        Err(err) => format!(
                            "Err:{}",
                            match err {
                                mlua::Error::CallbackError { cause, .. } => cause.to_string(),
                                e => e.to_string(),
                            }
                        ),
                    }
                }
            };

            let response_body = response.as_bytes();
            let response_len = response_body.len();
            let response_header = u16::to_be_bytes(response_len as u16);

            // header length is 2
            // header contains the length of the body
            let mut response = Vec::with_capacity(response_len + 2);
            response.append(&mut response_header.to_vec());
            response.append(&mut response_body.to_vec());

            stream.write_all(&response)?;
        }
    }
}
