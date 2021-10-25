use clap::clap_app;
use nog_iced::{
    iced::{self, Application, Color, Column, Command, Container, Text},
    load_font,
};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use rgb::Rgb;
use windows::Windows::Win32::{
    Foundation::HWND,
    UI::{
        KeyboardAndMouseInput::keybd_event,
        WindowsAndMessaging::{
            GetForegroundWindow, SetForegroundWindow, SetWindowLongW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
        },
    },
};

struct AppState {
    pub message: String,
    pub text_color: Color,
    pub bg: Color,
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
        Command::none()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        Container::new(Text::new(&self.state.message).color(self.state.text_color))
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
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
    let matches = clap_app! (nog_cli =>
        (version: "1.0")
        (author: "Tim Untersberger <timuntersberger2@gmail.com")
        (about: "Creates a nog notification")
        (@arg MESSAGE: -m --message +takes_value "The message of the notification. (Default: NogNotification)")
        (@arg COLOR: -b --bg_color +takes_value "The color of the notification. (Default: 0xFFFFFF)")
        (@arg TEXT_COLOR: -t --text_color +takes_value "The color of the notification text. (Default: 0x000000)")
        (@arg HEIGHT: -h --height +takes_value "The height of the notification. (Default: 100)")
        (@arg WIDTH: -w --width +takes_value "The width of the notification. (Default: 100)")
        (@arg X: -x --x +takes_value "The x position of the notification. (Default: 0)")
        (@arg Y: -y --y +takes_value "The y position of the notification. (Default: 0)")
        (@arg FONT_SIZE: -s --font_size +takes_value "The size of the notification text. (Default: 20)")
        (@arg FONT_NAME: -n --font_name +takes_value "The font of the notification text. (Default: Consolas)")
    )
    .get_matches();

    let message = matches.value_of("MESSAGE").unwrap_or("NogNotification");
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
    let height = matches
        .value_of("HEIGHT")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(100);
    let width = matches
        .value_of("WIDTH")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(100);
    let x = matches
        .value_of("X")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);
    let y = matches
        .value_of("Y")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);

    let font: &'static [u8] = Box::leak(Box::new(
        (*load_font(font_name.to_string())
            .or_else(|| load_font(String::from("Consolas")))
            .expect("The fallback font also failed? What?"))
        .clone(),
    ));

    let settings = iced::Settings {
        window: iced::window::Settings {
            always_on_top: true,
            decorations: false,
            position: iced::window::Position::Specific(x, y),
            size: (width, height),
            ..Default::default()
        },
        id: None,
        flags: AppState {
            message: String::from(message),
            text_color: Rgb::from_hex(text_color).0.into(),
            bg: Rgb::from_hex(color).0.into(),
        },
        default_font: Some(&font),
        default_text_size: font_size,
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
