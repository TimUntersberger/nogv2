use rgb::Rgb;
use std::io;
use std::mem;
use std::time::Duration;

use crate::InteractableItem;
use fuzzy_matcher::{skim::SkimMatcherV2 as Matcher, FuzzyMatcher};
use nog_client::{Client, ClientError};
use nog_iced::iced_native::subscription;
use nog_iced::{
    iced::{
        self, container,
        keyboard::{Event::KeyPressed, KeyCode, Modifiers},
        scrollable, text_input, Align, Application, Background, Color, Column, Command, Container,
        Length, Row, Scrollable, Space, Subscription, Text, TextInput,
    },
    iced_native::{window, Event},
};

#[derive(Debug, Clone)]
pub enum MenuMode {
    Files,
    ExecuteLua,
}

#[derive(Debug, Clone)]
pub enum Message {
    FilterChanged(String),
    KeyPressed(KeyCode, Modifiers),
    Exit,
}

#[derive(Default, Debug)]
pub struct State {
    pub items: Vec<Box<dyn InteractableItem>>,
    pub item_height: usize,
    pub hostname: String,
    pub port: String,
    pub color: Rgb,
    pub text_color: Rgb,
    pub max_visible_items: usize,
    /// Always contains the items fuzzy matched by the filter and sorted based on their score.
    pub filtered_items: Vec<Box<dyn InteractableItem>>,
    pub filter: String,
    pub selected_idx: usize,
}

pub struct App {
    state: State,
    client: Option<Client>,
    execute_output: String,
    mode: MenuMode,
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
                client: try_connect(&flags.hostname, &flags.port).ok(),
                state: flags,
                execute_output: String::new(),
                mode: MenuMode::Files,
                exit: false,
                filter_input_state: Default::default(),
                scrollable_state: Default::default(),
                matcher: Default::default(),
            },
            Command::none(),
        )
    }

    fn background_color(&self) -> Color {
        self.state.color.0.into()
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

                if self.state.filter.starts_with("$") {
                    self.mode = MenuMode::ExecuteLua;
                    let maybe_client = match mem::take(&mut self.client) {
                        Some(client) => Ok(client),
                        None => try_connect(&self.state.hostname, &self.state.port),
                    };

                    self.execute_output = maybe_client
                        .ok()
                        .and_then(|mut client| {
                            let code = &self.state.filter[1..];
                            let res = match client.execute_lua(code.to_string(), false) {
                                Ok(res) => res,
                                Err(ClientError::LuaExecutionFailed(msg)) => msg,
                                Err(ClientError::InvalidResponse(res)) => {
                                    format!("Invalid Response: {}", res)
                                }
                                Err(ClientError::IoError(_)) => return None,
                            };
                            self.client = Some(client);
                            Some(res)
                        })
                        .unwrap_or_else(|| {
                            String::from("network error: Failed to connect to the nog server")
                        });

                    let mut height = 50;

                    if !self.execute_output.is_empty() {
                        height += 5 + self
                            .execute_output
                            .split('\n')
                            .count()
                            .min(self.state.max_visible_items)
                            * self.state.item_height;
                    }

                    return Command::single(nog_iced::iced_native::command::Action::Window(
                        nog_iced::iced_native::window::Action::Resize {
                            width: 700,
                            // input height + vertical gap + result list height
                            height: height as u32,
                        },
                    ));
                } else {
                    self.mode = MenuMode::Files;
                    // A vec of the items that matched the filter and sorted based on their score.
                    let mut fuzzied_items = self
                        .state
                        .items
                        .iter()
                        .map(|i| {
                            (
                                i,
                                self.matcher.fuzzy_match(&i.get_text(), &self.state.filter),
                            )
                        })
                        .filter(|(_, score)| score.is_some())
                        .map(|(i, score)| ((i.clone(), score.unwrap())))
                        .collect::<Vec<(Box<dyn InteractableItem>, i64)>>();

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
            }
            Message::Exit => self.exit = true,
            Message::KeyPressed(key, mods) => match (key, mods) {
                (KeyCode::Escape, mods) if mods == Modifiers::empty() => {
                    self.exit = true;
                }
                (KeyCode::Enter, mods) if mods == Modifiers::empty() => {
                    match &self.mode {
                        MenuMode::Files => {
                            let item = &self.state.filtered_items[self.state.selected_idx];
                            item.on_submit();
                            self.exit = true;
                        },
                        MenuMode::ExecuteLua => {
                            self.exit = true;
                        },
                    }
                }
                (KeyCode::K | KeyCode::P, mods) if mods == Modifiers::CTRL => {
                    self.move_selection(-1);
                }
                (KeyCode::J | KeyCode::N, mods) if mods == Modifiers::CTRL => {
                    self.move_selection(1);
                }
                (KeyCode::W, mods) if mods == Modifiers::CTRL => {
                    let tokens = self.state.filter.split(' ').collect::<Vec<_>>();
                    let new_filter = if let [res @ .., _] = tokens.as_slice() {
                        res.join(" ")
                    } else {
                        String::from("")
                    };
                    return Command::perform((|| async { new_filter })(), Message::FilterChanged);
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
        let bg = self.state.color;
        let fg = self.state.text_color;

        let result_list = Scrollable::new(&mut self.scrollable_state)
            .scrollbar_width(10)
            .push(match self.mode {
                MenuMode::Files => Column::with_children(
                    self.state
                        .filtered_items
                        .iter()
                        .enumerate()
                        .map(|(i, item)| {
                            let text = Text::new(item.get_text());

                            let content = Row::new()
                                .align_items(Align::Center)
                                .push(Space::with_width(Length::Units(3)))
                                .push(text);

                            Container::new(content)
                                .style(MenuItemStyle {
                                    is_selected: i == selected_idx,
                                    bg,
                                    fg,
                                })
                                .align_y(Align::Center)
                                .height(Length::Units(item_height))
                                .width(Length::Fill)
                                .into()
                        })
                        .collect(),
                ),
                MenuMode::ExecuteLua => Column::new()
                    .padding(5)
                    .push(Text::new(&self.execute_output).color(fg.0)),
            });

        let filter_input = TextInput::new(
            &mut self.filter_input_state,
            "Search...",
            &self.state.filter,
            Message::FilterChanged,
        )
        .style(FilterInputStyle { bg, fg })
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
        nog_iced::run::<Self>(settings, None)
    }
}

fn try_connect(hostname: &str, port: &str) -> io::Result<Client> {
    Client::connect(
        String::from(&format!("{}:{}", hostname, port)),
        Some(Duration::from_millis(1)),
    )
}

struct MenuItemStyle {
    pub is_selected: bool,
    pub bg: Rgb,
    pub fg: Rgb,
}

impl container::StyleSheet for MenuItemStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(if self.is_selected {
                self.bg.scaled(1.3).0.into()
            } else {
                self.bg.0.into()
            })),
            text_color: Some(self.fg.0.into()),
            ..Default::default()
        }
    }
}

struct FilterInputStyle {
    pub bg: Rgb,
    pub fg: Rgb,
}

impl text_input::StyleSheet for FilterInputStyle {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(self.bg.0.into()),
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
        self.fg.0.into()
    }

    fn selection_color(&self) -> Color {
        Color::from_rgb(0.8, 0.8, 1.0)
    }
}
