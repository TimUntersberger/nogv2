use mlua::Error;
use rustyline::Editor;

use super::LuaRuntime;

pub fn start(rt: &mut LuaRuntime) {
    let mut editor = Editor::<()>::new();

    loop {
        let mut prompt = "> ";
        let mut line = String::new();

        loop {
            match editor.readline(prompt) {
                Ok(input) => line.push_str(&input),
                Err(_) => return,
            }

            match rt.eval(&line) {
                Ok(value) => {
                    editor.add_history_entry(line);
                    println!("{:#?}", value);
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
                        },
                        _ => eprintln!("error: {}", e)
                    }
                    break;
                }
            }
        }
    }
}
