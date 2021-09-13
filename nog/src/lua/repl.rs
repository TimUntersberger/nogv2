use std::sync::mpsc::{sync_channel, Sender};

use mlua::Error;
use rustyline::Editor;

use crate::action::{Action, ExecuteLuaActionFn};
use crate::event::Event;

use super::LuaRuntime;

pub fn start(tx: Sender<Event>) {
    let mut editor = Editor::<()>::new();

    loop {
        let mut prompt = "> ";
        let mut line = String::new();

        loop {
            match editor.readline(prompt) {
                Ok(input) => line.push_str(&input),
                Err(_) => return,
            }

            let (result_tx, result_rx) = sync_channel(1);

            tx.send(Event::Action(Action::ExecuteLua {
                code: line.clone(),
                capture_stdout: false,
                cb: ExecuteLuaActionFn::new(move |res| {
                    result_tx.send(res).unwrap();
                }),
            }))
            .unwrap();

            match result_rx.recv().unwrap() {
                Ok(output) => {
                    editor.add_history_entry(line);
                    println!("{}", output);
                    break;
                }
                Err(Error::SyntaxError {
                    incomplete_input: true,
                    ..
                }) => {
                    line.push_str("\n");
                    prompt = ">>";
                }
                Err(e) => {
                    editor.add_history_entry(line);
                    match e {
                        Error::CallbackError { traceback, cause } => {
                            eprintln!("error: {}\n{}", cause, traceback);
                        }
                        _ => eprintln!("error: {}", e),
                    }
                    break;
                }
            }
        }
    }
}

pub fn spawn(tx: Sender<Event>) {
    std::thread::spawn(move || {
        start(tx);
    });
}
