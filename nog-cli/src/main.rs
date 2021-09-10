use nog_client::{Client, ClientError};
use rustyline::Editor;

fn repl(client: &mut Client) {
    let mut editor = Editor::<()>::new();

    loop {
        let mut prompt = "> ";
        let mut line = String::new();

        loop {
            match editor.readline(prompt) {
                Ok(input) => line.push_str(&input),
                Err(_) => return,
            }

            if line == "\\reconnect" {
                if let Err(e) = client.reconnect() {
                    eprintln!("error: {}", e);
                    break;
                }

                println!("Reconnected to the server!");

                break;
            }

            match client.execute_lua(line.clone()) {
                Ok(output) => {
                    editor.add_history_entry(line);
                    println!("{}", output);
                    break;
                }
                Err(e) => {
                    editor.add_history_entry(line);
                    match e {
                        ClientError::IoError(e) => eprintln!("network error: {}", e),
                        ClientError::LuaExecutionFailed(msg) => eprintln!("lua error: {}", msg),
                        ClientError::InvalidResponse(res) => {
                            eprintln!("response has invalid format: '{}'", res)
                        }
                    }
                    break;
                }
            };
        }
    }
}

fn main() {
    let addr = String::from("localhost:8080");
    let mut client = match Client::connect(addr.clone()) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("error: {}", e);
            return;
        }
    };

    println!("Connected to the server!");
    dbg!(client.get_bar_content());
    // repl(&mut client);
}
