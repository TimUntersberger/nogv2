use iced::{Application, Color, Command, Container, Row, Text};

#[derive(Debug)]
enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
struct Item {
    text: String,
    fg: Color,
    bg: Color,
    alignment: Alignment,
}

#[derive(Default, Debug)]
struct AppState {
    bg: Color,
    items: Vec<Item>,
}

struct App {
    state: AppState,
}

impl Application for App {
    type Executor = iced::executor::Default;

    type Message = ();

    type Flags = AppState;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self { state: flags }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Nog Bar")
    }

    fn background_color(&self) -> iced::Color {
        self.state.bg
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        todo!()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let mut left_items = Row::new();
        let mut center_items = Row::new();
        let mut right_items = Row::new();

        for item in &self.state.items {
            let new_item = Container::new(Text::new(&item.text)).style(Style {
                fg: item.fg,
                bg: item.bg,
            });

            match item.alignment {
                Alignment::Left => left_items = left_items.push(new_item),
                Alignment::Center => center_items = center_items.push(new_item),
                Alignment::Right => right_items = right_items.push(new_item),
            };
        }

        let left = Container::new(left_items)
            .align_x(iced::Align::Start)
            .height(iced::Length::Fill)
            .width(iced::Length::Fill);

        let center = iced::Container::new(center_items)
            .align_x(iced::Align::Center)
            .height(iced::Length::Fill)
            .width(iced::Length::Fill);

        let right = iced::Container::new(right_items)
            .align_x(iced::Align::End)
            .height(iced::Length::Fill)
            .width(iced::Length::Fill);

        Row::new()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .push(left)
            .push(center)
            .push(right)
            .padding(1)
            .into()
    }
}

fn main() {
    let settings = iced::Settings {
        window: iced::window::Settings {
            always_on_top: true,
            decorations: false,
            position: iced::window::Position::Specific(0, 0),
            size: (1920, 1),
            ..Default::default()
        },
        flags: AppState {
            bg: Color::BLACK,
            items: vec![
                Item {
                    alignment: Alignment::Left,
                    text: String::from(" 1 "),
                    fg: Color::WHITE,
                    bg: Color::from_rgb(0.247, 0.247, 0.247),
                },
                Item {
                    alignment: Alignment::Left,
                    text: String::from(" 2 "),
                    fg: Color::WHITE,
                    bg: Color::from_rgb(0.247, 0.247, 0.247),
                },
                Item {
                    alignment: Alignment::Left,
                    text: String::from(" 3 "),
                    fg: Color::WHITE,
                    bg: Color::from_rgb(0.247, 0.247, 0.247),
                },
                Item {
                    alignment: Alignment::Left,
                    text: String::from(" "),
                    fg: Color::WHITE,
                    bg: Color::BLACK,
                },
                Item {
                    alignment: Alignment::Left,
                    text: String::from("test"),
                    fg: Color::WHITE,
                    bg: Color::BLACK,
                },
                Item {
                    alignment: Alignment::Center,
                    text: String::from("center"),
                    fg: Color::WHITE,
                    bg: Color::BLACK,
                },
                Item {
                    alignment: Alignment::Right,
                    text: String::from("right"),
                    fg: Color::WHITE,
                    bg: Color::BLACK,
                },
            ],
        },
        ..Default::default()
    };

    App::run(settings);
}

struct Style {
    fg: Color,
    bg: Color,
}

impl iced::container::StyleSheet for Style {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            text_color: Some(self.fg),
            background: Some(iced::Background::Color(self.bg)),
            ..Default::default()
        }
    }
}
