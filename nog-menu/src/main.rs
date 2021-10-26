#![feature(windows_process_extensions_raw_arg)]

use clap::clap_app;
use dyn_clone::{clone_trait_object, DynClone};
use nog_iced::iced::{
    self,
    window::{self, Position},
    Application, Color, Settings,
};
use rgb::Rgb;
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
    let matches = clap_app! (nog_cli =>
        (version: "1.0")
        (author: "Tim Untersberger <timuntersberger2@gmail.com")
        (about: "A menu similiar to spotlight on macos. Also supports executing lua code in nog")
        (@arg HOSTNAME: -h --hostname +takes_value "The hostname of the nog server. (Default: localhost)")
        (@arg PORT: -p --port +takes_value "The port of the nog server. (Default: 8080)")
        (@arg COLOR: -b --bg_color +takes_value "The color of the menu. (Default: 0xFFFFFF)")
        (@arg TEXT_COLOR: -t --text_color +takes_value "The color of the menu text. (Default: 0x000000)")
        (@arg FONT_SIZE: -s --font_size +takes_value "The size of the menu text. (Default: 20)")
        (@arg FONT_NAME: -n --font_name +takes_value "The font of the menu text. (Default: Consolas)")
    )
    .get_matches();

    let hostname = matches
        .value_of("HOSTNAME")
        .unwrap_or("localhost")
        .to_string();
    let port = matches.value_of("PORT").unwrap_or("8080").to_string();
    let color = matches
        .value_of("COLOR")
        .and_then(|v| i32::from_str_radix(v.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0xFFFFFF);
    let text_color = matches
        .value_of("TEXT_COLOR")
        .and_then(|v| i32::from_str_radix(v.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x000000);
    let font_name = matches.value_of("FONT_NAME").unwrap_or("Consolas");
    let font_size = matches
        .value_of("FONT_SIZE")
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(20);

    let items = Api::get_files()
        .drain(..)
        .map(Box::new)
        .map(|x| Box::<dyn InteractableItem>::from(x))
        .collect();

    let font: &'static [u8] = Box::leak(Box::new(
        (*nog_iced::load_font(font_name.to_string())
            .or_else(|| nog_iced::load_font(String::from("Consolas")))
            .expect("The fallback font also failed? What?"))
        .clone(),
    ));

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
        default_text_size: font_size,
        default_font: Some(font),
        flags: ui::State {
            items,
            hostname,
            port,
            color: Rgb::from_hex(color),
            text_color: Rgb::from_hex(text_color),
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
