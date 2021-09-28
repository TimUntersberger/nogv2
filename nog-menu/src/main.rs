#![feature(windows_process_extensions_raw_arg)]

use std::path::PathBuf;

use dyn_clone::{clone_trait_object, DynClone};
use nog_iced::iced::{
    window::{self, Position},
    Application, Settings,
};
use win::Win as Api;

mod ui;
mod win;

pub trait Native {
    fn get_files() -> Vec<ResultItem>;
    fn start_program(path: &str);
}

pub trait InteractableItem: std::fmt::Debug + DynClone {
    fn get_text(&self) -> &str;
    fn on_submit(&self);
    //TODO: add support for item specific keybindings
}

clone_trait_object!(InteractableItem);

#[derive(Debug, Clone)]
pub struct ResultItem {
    pub path: String,
    pub name: String,
}

impl InteractableItem for ResultItem {
    fn get_text(&self) -> &str {
        &self.name
    }

    fn on_submit(&self) {
        Api::start_program(&format!("{}{}{}", self.path, "\\", self.name));
    }
}

fn main() {
    let map = pelite::FileMap::open(r#"C:\Users\Tim\Desktop\neovide.exe"#).unwrap();
    let file = pelite::PeFile::from_bytes(&map).unwrap();
	let resources = file.resources().expect("Error binary does not have resources");
	for (_, group) in resources.icons().filter_map(Result::ok) {
		let mut ico_bytes = Vec::new();
		group.write(&mut ico_bytes).unwrap();
	}
}

fn main2() {
    let items = Api::get_files()
        .drain(..)
        .map(Box::new)
        .map(|x| Box::<dyn InteractableItem>::from(x))
        .collect();

    ui::App::run(Settings {
        window: window::Settings {
            decorations: false,
            resizable: false,
            always_on_top: true,
            position: Position::Centered,
            transparent: true,
            size: (700, 50),
            ..Default::default()
        },
        default_text_size: 20,
        flags: ui::State {
            items,
            max_visible_items: 5,
            item_height: 32,
            filtered_items: Vec::new(),
            selected_idx: 0,
            filter: String::from(""),
        },
        ..Default::default()
    })
    .unwrap();
}
