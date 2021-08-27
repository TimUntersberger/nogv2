use nog_protocol::Message;
use rustyline::Editor;
use std::{io::{Read, Write}, net::TcpStream, time::Duration};

struct Client {
    stream: TcpStream
}

impl Client {
    pub fn connect(addr: String) -> Self {
        let stream = TcpStream::connect(addr).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        stream.set_write_timeout(Some(Duration::from_secs(2))).unwrap();

        Self {
            stream
        }
    }

    pub fn send_message(&mut self, msg: &Message) -> String {
        self.stream
            .write(&msg.serialize())
            .unwrap();

        let mut response_header = [0u8; 2];
        self.stream.read_exact(&mut response_header).unwrap();
        let response_len = u16::from_be_bytes(response_header);

        let mut response_body = vec![0u8; response_len as usize];
        self.stream.read_exact(&mut response_body).unwrap();

        String::from_utf8(response_body).unwrap()
    }
}

fn main() {
    let addr = String::from("localhost:8080");
    let mut client = Client::connect(addr);
    let mut editor = Editor::<()>::new();

    loop {
        let mut prompt = "> ";
        let mut line = String::new();

        loop {
            match editor.readline(prompt) {
                Ok(input) => line.push_str(&input),
                Err(_) => return,
            }

            let msg = Message::ExecuteLua {
                code: line.clone(),
            };

            let response = client.send_message(&msg);

            if let Some(tokens) = response.split_once(":") {
                match tokens {
                    ("Ok", output) => {
                        editor.add_history_entry(line);
                        println!("{}", output);
                        break;
                    },
                    ("Err", msg) => {
                        editor.add_history_entry(line);
                        eprintln!("error: {}", msg);
                        break;
                    },
                    _ => {}
                }
            }
        }
    }

}
