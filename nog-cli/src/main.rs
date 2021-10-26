use std::io;

use clap::clap_app;
use crossterm::terminal::enable_raw_mode;
use nog_client::{json, Client, ClientError};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Terminal,
};

fn main() {
    let matches = clap_app! (nog_cli =>
        (@setting SubcommandRequiredElseHelp)
        (version: "1.0")
        (author: "Tim Untersberger <timuntersberger2@gmail.com")
        (about: "Communicate with nog via the command line")
        (@arg HOSTNAME: -h --hostname +takes_value "The hostname of the nog server. (Default: localhost)")
        (@arg PORT: -p --port +takes_value "The port of the nog server. (Default: 8080)")
        (@subcommand execute =>
            (about: "Execute an arbitrary lua string in the context of the lua runtime")
            (version: "1.0")
            (author: "Tim Untersberger <timuntersberger2@gmail.com")
            (@arg code: +required "The lua code string to be executed")
        )
        (@subcommand state =>
            (about: "Prints the current state of nog")
            (version: "1.0")
            (author: "Tim Untersberger <timuntersberger2@gmail.com")
        )
        (@subcommand render =>
            (about: "Tries to render the currently managed windows in the terminal")
            (version: "1.0")
            (author: "Tim Untersberger <timuntersberger2@gmail.com")
        )
        (@subcommand bar =>
            (about: "Prints the current state of the nog-bar")
            (version: "1.0")
            (author: "Tim Untersberger <timuntersberger2@gmail.com")
        )
        (@subcommand render_bar =>
            (about: "Tires to render the current state of the nog-bar")
            (version: "1.0")
            (author: "Tim Untersberger <timuntersberger2@gmail.com")
        )
    )
    .get_matches();

    let hostname = matches.value_of("HOSTNAME").unwrap_or("localhost");
    let port = matches.value_of("PORT").unwrap_or("8080");

    let addr = String::from(format!("{}:{}", hostname, port));

    let mut client = match Client::connect(addr, None) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("error: {}", e);
            return;
        }
    };

    // println!("Connected to the server!");

    match matches.subcommand() {
        ("execute", Some(m)) => {
            let code = m.value_of("code").unwrap_or_default();
            match client.execute_lua(code.to_string(), false) {
                Ok(output) => println!("{}", output),
                Err(e) => eprintln!("error: {:?}", e),
            };
        }
        ("state", Some(m)) => {
            println!(
                "{}",
                json::to_string_pretty(&client.get_state().unwrap()).unwrap()
            );
        }
        ("bar", Some(m)) => todo!(),
        ("render", Some(m)) => tui(),
        ("render_bar", Some(m)) => todo!(),
        _ => unreachable!("It shouldn't be possible to provide an invalid subcommand name"),
    }
}

fn tui() {
    enable_raw_mode().unwrap();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    terminal
        .draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, chunks[0]);
            let block = Block::default().title("Block 2").borders(Borders::ALL);
            f.render_widget(block, chunks[1]);
        })
        .unwrap();
}
