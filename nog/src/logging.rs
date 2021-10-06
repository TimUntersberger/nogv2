use std::sync::mpsc::channel;
use std::thread;

pub fn init() -> Result<(), log::SetLoggerError> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            print!("{}", msg);
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
