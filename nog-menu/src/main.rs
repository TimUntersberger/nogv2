use crate::iced::keyboard::Event::KeyPressed;
use fuzzy_matcher::{skim::SkimMatcherV2 as Matcher, FuzzyMatcher};
use nog_iced::iced_native::subscription;
use nog_iced::{
    iced::{
        self, container,
        keyboard::{KeyCode, Modifiers},
        scrollable, text_input,
        window::Position,
        Align, Application, Background, Color, Column, Command, Container, Length, Row, Scrollable,
        Settings, Space, Subscription, Text, TextInput,
    },
    iced_native::{window, Event},
};
use std::sync::mpsc::{sync_channel, SyncSender};

#[derive(Debug, Clone)]
enum Message {
    FilterChanged(String),
    KeyPressed(KeyCode, Modifiers),
    Exit,
}

#[derive(Default, Debug)]
struct State {
    items: Vec<ResultItem>,
    item_height: usize,
    max_visible_items: usize,
    /// Always contains the items fuzzy matched by the filter and sorted based on their score.
    filtered_items: Vec<ResultItem>,
    filter: String,
    selected_idx: usize,
}

struct App {
    state: State,
    exit: bool,
    filter_input_state: text_input::State,
    scrollable_state: scrollable::State,
    matcher: Matcher,
}

impl App {
    pub fn move_selection(&mut self, by: isize) {
        self.state.selected_idx = std::cmp::max(self.state.selected_idx as isize + by, 0) as usize;

        if self.state.selected_idx >= self.state.items.len() {
            self.state.selected_idx = self.state.items.len() - 1;
        }
    }
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;

    type Flags = State;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                state: flags,
                exit: false,
                filter_input_state: Default::default(),
                scrollable_state: Default::default(),
                matcher: Default::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("nog_menu")
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::FilterChanged(new_value) => {
                self.state.filter = new_value;
                self.state.selected_idx = 0;
                // A vec of the items that matched the filter and sorted based on their score.
                let mut fuzzied_items = self
                    .state
                    .items
                    .iter()
                    .map(|i| (i, self.matcher.fuzzy_match(&i.name, &self.state.filter)))
                    .filter(|(_, score)| score.is_some())
                    .map(|(i, score)| ((i.clone(), score.unwrap())))
                    .collect::<Vec<(ResultItem, i64)>>();

                fuzzied_items.sort_by_key(|(_, score)| *score);

                self.state.filtered_items = fuzzied_items.into_iter().map(|(x, _)| x).collect();

                return Command::single(nog_iced::iced_native::command::Action::Window(
                    nog_iced::iced_native::window::Action::Resize {
                        width: 700,
                        // input height + vertical gap + result list height
                        height: 50
                            + 5
                            + self.state.item_height as u32
                                * self
                                    .state
                                    .filtered_items
                                    .len()
                                    .min(self.state.max_visible_items)
                                    as u32,
                    },
                ));
            }
            Message::Exit => self.exit = true,
            Message::KeyPressed(key, mods) => match (key, mods) {
                (KeyCode::Escape, mods) if mods == Modifiers::empty() => {
                    self.exit = true;
                }
                (KeyCode::K | KeyCode::P, mods) if mods == Modifiers::CTRL => {
                    self.move_selection(-1);
                }
                (KeyCode::J | KeyCode::N, mods) if mods == Modifiers::CTRL => {
                    self.move_selection(1);
                }
                (KeyCode::W, mods) if mods == Modifiers::CTRL => {
                    let tokens = self.state.filter.split(' ').collect::<Vec<_>>();
                    if let [res @ .., _] = tokens.as_slice() {
                        self.state.filter = res.join(" ");
                    } else {
                        self.state.filter = String::from("");
                    }
                }
                _ => {}
            },
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::events_with::<Self::Message>(move |event, _| match event {
            Event::Window(event) => match event {
                window::Event::Unfocused => Some(Message::Exit),
                _ => None,
            },
            Event::Keyboard(event) => match event {
                KeyPressed {
                    key_code,
                    modifiers,
                } => Some(Message::KeyPressed(key_code, modifiers)),
                _ => None,
            },
            _ => None,
        })
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        self.filter_input_state.focus();

        let selected_idx = self.state.selected_idx;
        let item_height = self.state.item_height as u16;

        let result_list = Scrollable::new(&mut self.scrollable_state)
            .scrollbar_width(10)
            .push(Column::with_children(
                self.state
                    .filtered_items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let title = Text::new(item.name.clone());

                        let content = Row::new()
                            .align_items(Align::Center)
                            .push(Space::with_width(Length::Units(3)))
                            .push(title);

                        Container::new(content)
                            .style(MenuItemStyle {
                                is_selected: i == selected_idx,
                            })
                            .align_y(Align::Center)
                            .height(Length::Units(item_height))
                            .width(Length::Fill)
                            .into()
                    })
                    .collect(),
            ));

        let filter_input = TextInput::new(
            &mut self.filter_input_state,
            "Search...",
            &self.state.filter,
            Message::FilterChanged,
        )
        .style(FilterInputStyle)
        .size(30)
        .padding(10);

        Column::new()
            .push(filter_input)
            .push(Space::with_height(Length::Units(5)))
            .push(result_list)
            .into()
    }

    fn run(settings: iced::Settings<Self::Flags>) -> iced::Result
    where
        Self: 'static,
    {
        nog_iced::run::<Self>(
            settings,
            Some(Box::new(|w| {
                dbg!(w);
            })),
        )
    }
}

#[derive(Debug, Clone)]
struct ResultItem {
    pub path: String,
    pub name: String,
}

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
    let path = String::from(format!(r#"C:\Users\{}\Desktop"#, "Tim"));
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

fn fetch_program_files(is86: bool, dir: Option<String>) {
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
                    fetch_program_files(is86, Some(path));
                } else if name.ends_with(".exe") {
                    dbg!(path);
                }
            }
        }
    }
}

fn main() {
    let (tx, rx) = sync_channel(10);
    // println!("desktop");
    // fetch_desktop_programs();
    // println!("program files");
    // fetch_program_files(false, None);
    // println!("start menu programs");

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

    App::run(Settings {
        window: iced::window::Settings {
            decorations: false,
            resizable: false,
            always_on_top: true,
            position: Position::Centered,
            transparent: true,
            size: (700, 50),
            ..Default::default()
        },
        default_text_size: 20,
        flags: State {
            items,
            max_visible_items: 5,
            item_height: 32,
            filtered_items: Vec::new(),
            selected_idx: 0,
            filter: String::from(""),
        },
        ..Default::default()
    }).unwrap();
}

struct MenuItemStyle {
    pub is_selected: bool,
}

impl container::StyleSheet for MenuItemStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(if self.is_selected {
                Color::new(0.8, 0.8, 0.8, 1.0)
            } else {
                Color::WHITE
            })),
            ..Default::default()
        }
    }
}

struct FilterInputStyle;

impl text_input::StyleSheet for FilterInputStyle {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(Color::WHITE),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: Color::from_rgb(0.5, 0.5, 0.5),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn value_color(&self) -> Color {
        Color::from_rgb(0.3, 0.3, 0.3)
    }

    fn selection_color(&self) -> Color {
        Color::from_rgb(0.8, 0.8, 1.0)
    }
}
