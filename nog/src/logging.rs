use std::sync::mpsc::channel;
use std::thread;

#[cfg(not(debug_assertions))]
use chrono::Utc;
#[cfg(not(debug_assertions))]
use std::fs::OpenOptions;
#[cfg(not(debug_assertions))]
use std::io::prelude::*;
#[cfg(not(debug_assertions))]
use crate::paths::get_config_path;

pub fn init() -> Result<(), log::SetLoggerError> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        #[cfg(not(debug_assertions))]
        let mut file = OpenOptions::new()
            .append(true)
            .open({
                let mut path = get_config_path();

                path.push("logs");
                path.set_file_name(Utc::now().format("%Y_%m_%d").to_string());
                path.set_extension(".txt");

                path
            })
            .unwrap();

        while let Ok(msg) = rx.recv() {
            print!("{}", &msg);
            #[cfg(not(debug_assertions))]
            write!(file, "{}", &msg).unwrap();
        }
    });

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .level_for("nog::keybinding_event_loop", log::LevelFilter::Info)
        .chain(tx)
        .apply()?;
    Ok(())
}
