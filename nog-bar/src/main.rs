use iced::{Application, Color, Command, Container, Row, Text};
use nog_client::{BarItem, BarItemAlignment, Client};
use nog_iced::{iced, load_font};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::time::Duration;
use windows::Windows::Win32::Foundation::HWND;
use windows::Windows::Win32::UI::KeyboardAndMouseInput::keybd_event;
use windows::Windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, SetForegroundWindow, SetWindowLongW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
};

#[derive(Debug)]
struct AppState {
    client: Client,
    bg: Color,
    items: Vec<BarItem>,
}

struct App {
    state: AppState,
    exit: bool,
}

impl Application for App {
    type Executor = iced::executor::Default;

    type Message = ();

    type Flags = AppState;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                state: flags,
                exit: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("nog_bar")
    }

    fn background_color(&self) -> iced::Color {
        self.state.bg
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        match self.state.client.get_bar_content() {
            Ok(bar_content) => {
                self.state.bg = bar_content.bg.into();
                self.state.items = bar_content.items;
            }
            Err(_) => {
                self.exit = true;
            }
        };
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(Duration::from_millis(10)).map(|_| ())
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let mut left_items = Row::new();
        let mut center_items = Row::new();
        let mut right_items = Row::new();

        for item in &self.state.items {
            let new_item = Container::new(Text::new(&item.text)).style(Style {
                fg: item.fg.into(),
                bg: item.bg.into(),
            });

            match item.alignment {
                BarItemAlignment::Left => left_items = left_items.push(new_item),
                BarItemAlignment::Center => center_items = center_items.push(new_item),
                BarItemAlignment::Right => right_items = right_items.push(new_item),
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

    fn run(settings: iced::Settings<Self::Flags>) -> iced::Result
    where
        Self: 'static,
    {
        //TODO: platform specific
        let prev_hwnd = unsafe { GetForegroundWindow() };

        nog_iced::run::<Self>(
            settings,
            Some(Box::new(move |w| match &w.raw_window_handle() {
                RawWindowHandle::Windows(win_hndl) => unsafe {
                    let hwnd = HWND(win_hndl.hwnd as isize);
                    SetWindowLongW(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32);
                    keybd_event(0, 0, Default::default(), 0);
                    SetForegroundWindow(prev_hwnd);
                },
                handle => todo!("not supported yet: {:?}", handle),
            })),
        )
    }
}

fn main() {
    let mut client = Client::connect("localhost:8080".into()).unwrap();
    let bar_content = client.get_bar_content().unwrap();
    let font: &'static [u8] = Box::leak(Box::new(
        (*load_font(bar_content.font_name)
            .or_else(|| load_font(String::from("Consolas")))
            .expect("The fallback font also failed? What?"))
        .clone(),
    ));
    let settings = iced::Settings {
        window: iced::window::Settings {
            always_on_top: true,
            decorations: false,
            position: iced::window::Position::Specific(0, 0),
            size: (1920, bar_content.height as u32),
            ..Default::default()
        },
        id: None,
        flags: AppState {
            client,
            bg: bar_content.bg.into(),
            items: bar_content.items,
        },
        default_font: Some(&font),
        default_text_size: bar_content.font_size as u16,
        text_multithreading: false,
        antialiasing: true,
        exit_on_close_request: true,
    };

    App::run(settings).expect("Failed to start nog-bar");
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
