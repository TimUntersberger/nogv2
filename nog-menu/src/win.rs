use crate::{Native, ResultItem};
use std::{
    os::windows::process::CommandExt,
    process::Command,
    sync::mpsc::{sync_channel, SyncSender},
};

fn fetch_start_menu_programs(tx: SyncSender<Option<ResultItem>>, dir: Option<String>) {
    let start_menu_path = String::from(r#"C:\ProgramData\Microsoft\Windows\Start Menu\Programs"#);
    let path = dir.unwrap_or(start_menu_path);
    let dir_items = std::fs::read_dir(path.clone()).unwrap();

    for dir_item in dir_items {
        if let Ok(dir_item) = dir_item {
            let metadata = dir_item.metadata().unwrap();
            let name = dir_item.file_name().into_string().unwrap();
            if metadata.is_dir() {
                fetch_start_menu_programs(tx.clone(), Some(format!("{}\\{}", &path, name)));
            } else if name.ends_with(".lnk") {
                tx.send(Some(ResultItem {
                    path: path.clone(),
                    name,
                }))
                .unwrap();
            }
        }
    }
}

fn fetch_desktop_programs(tx: SyncSender<Option<ResultItem>>) {
    let path = String::from(format!(r#"C:\Users\{}\Desktop"#, "timun"));
    let dir_items = std::fs::read_dir(path.clone()).unwrap();

    for dir_item in dir_items {
        if let Ok(dir_item) = dir_item {
            let metadata = dir_item.metadata().unwrap();
            let name = dir_item.file_name().into_string().unwrap();

            if metadata.is_file() && (name.ends_with(".lnk") || name.ends_with(".exe")) {
                tx.send(Some(ResultItem {
                    path: path.clone(),
                    name,
                }))
                .unwrap();
            }
        }
    }
}

fn fetch_program_files(tx: SyncSender<Option<ResultItem>>, is86: bool, dir: Option<String>) {
    let path = String::from(if is86 {
        r#"C:\Program Files (x86)"#
    } else {
        r#"C:\Program Files"#
    });
    let path = dir.unwrap_or(path);

    if let Ok(dir_items) = std::fs::read_dir(path.clone()) {
        for dir_item in dir_items {
            if let Ok(dir_item) = dir_item {
                let metadata = dir_item.metadata().unwrap();
                let name = dir_item.file_name().into_string().unwrap();
                let path = format!("{}\\{}", &path, name);
                if metadata.is_dir() {
                    fetch_program_files(tx.clone(), is86, Some(path));
                } else if name.ends_with(".exe") {
                    dbg!(path);
                }
            }
        }
    }
}

pub struct Win;

impl Native for Win {
    fn get_files() -> Vec<crate::ResultItem> {
        let (tx, rx) = sync_channel(10);
        // println!("desktop");
        // fetch_desktop_programs();
        // println!("program files");
        // fetch_program_files(false, None);
        // println!("start menu programs");
        //
        // {
        //     let tx = tx.clone();
        //     std::thread::spawn(move || {
        //         fetch_program_files(tx.clone(), false, None);
        //         tx.send(None).unwrap();
        //     });
        // }

        {
            let tx = tx.clone();
            std::thread::spawn(move || {
                fetch_start_menu_programs(tx.clone(), None);
                tx.send(None).unwrap();
            });
        }

        {
            let tx = tx.clone();
            std::thread::spawn(move || {
                fetch_desktop_programs(tx.clone());
                tx.send(None).unwrap();
            });
        }

        // How many functions are currently searching for programs.
        let max_done_count = 2;

        // Once this variable is equal to the `max_done_count` we expect that we won't receive any more
        // resultitems.
        let mut done_count = 0;

        let mut items = Vec::new();

        for item in rx {
            match item {
                Some(item) => {
                    items.push(item);
                }
                None => {
                    done_count += 1;
                    if done_count == max_done_count {
                        break;
                    }
                }
            }
        }

        items
    }

    fn start_program(path: &str) {
        Command::new("cmd").arg("/C").raw_arg(&format!(r#"start "" "{}""#, path)).spawn().unwrap();
    }
}
