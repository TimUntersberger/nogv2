use nog_iced::iced;
use iced::{Row, Application, Command};

struct App;

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = ();

    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self, Command::none())
    }

    fn title(&self) -> String {
        String::from("nog_menu")
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        Row::new()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(1)
            .into()
    }

    fn run(settings: iced::Settings<Self::Flags>) -> iced::Result
    where
        Self: 'static,
    {
        nog_iced::run::<Self>(settings, Some(Box::new(|w| {
            dbg!(w);
        })))
    }
}

fn main() {
    App::run(Default::default());
}
