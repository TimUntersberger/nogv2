use iced::{Application, Color, Command, Element, Menu, Subscription, window};

pub struct Instance<A: Application>(A);

#[cfg(not(target_arch = "wasm32"))]
impl<A> iced_winit::Program for Instance<A>
where
    A: Application,
{
    type Renderer = crate::renderer::Renderer;
    type Message = A::Message;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.0.update(message)
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.0.view()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<A> iced_winit::Application for Instance<A>
where
    A: Application,
{
    type Flags = A::Flags;

    fn new(flags: Self::Flags) -> (Self, Command<A::Message>) {
        let (app, command) = A::new(flags);

        (Instance(app), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn mode(&self) -> iced_winit::Mode {
        match self.0.mode() {
            window::Mode::Windowed => iced_winit::Mode::Windowed,
            window::Mode::Fullscreen => iced_winit::Mode::Fullscreen,
            window::Mode::Hidden => iced_winit::Mode::Hidden,
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.0.subscription()
    }

    fn background_color(&self) -> Color {
        self.0.background_color()
    }

    fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }

    fn should_exit(&self) -> bool {
        self.0.should_exit()
    }

    fn menu(&self) -> Menu<Self::Message> {
        self.0.menu()
    }
}
