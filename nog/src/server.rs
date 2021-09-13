use crate::action::ExecuteLuaActionFn;
use crate::{action::Action, event::Event, thread_safe::ThreadSafe};
use log::error;
use nog_protocol::{BarContent, BarItem, BarItemAlignment, Message};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{sync_channel, Sender},
        Arc, RwLock,
    },
};

pub struct Server {
    tx: Sender<Event>,
    bar_content: ThreadSafe<BarContent>,
    host: String,
    port: u32,
}

impl Server {
    pub fn new(tx: Sender<Event>, bar_content: ThreadSafe<BarContent>) -> Self {
        Self {
            tx,
            bar_content,
            host: "localhost".into(),
            port: 8080,
        }
    }
    pub fn spawn(tx: Sender<Event>, bar_content: ThreadSafe<BarContent>) {
        std::thread::spawn(move || {
            let server = Server::new(tx, bar_content);
            server.start();
        });
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).unwrap();

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let tx = self.tx.clone();
                let bar_content = self.bar_content.clone();
                std::thread::spawn(move || {
                    if let Err(_e) = handle_client(stream, tx, bar_content) {
                        // error!("{:?}", e);
                    }
                });
            }
        }
    }
}

fn handle_client(
    mut stream: TcpStream,
    tx: Sender<Event>,
    bar_content: ThreadSafe<BarContent>,
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
                Message::GetBarContent => serde_json::to_string(&*bar_content.read())
                    .expect("Serde failed to serialize the bar content"),
                Message::ExecuteLua { code } => {
                    let (result_tx, result_rx) = sync_channel(1);

                    tx.send(Event::Action(Action::ExecuteLua {
                        code,
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

            stream.write(&response)?;
        }
    }
}
