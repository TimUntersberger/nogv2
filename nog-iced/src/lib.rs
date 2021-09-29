pub use iced;
pub use iced_native;

use font_kit::properties::Properties;
use font_kit::family_name::FamilyName;
use font_kit::source::SystemSource;
use iced::{Application, Settings};
use iced_wgpu as renderer;
use instance::Instance;
use std::sync::Arc;

mod iced_run;
mod instance;

pub fn run<S: 'static + Application>(
    settings: Settings<S::Flags>,
    after_window_created: Option<Box<dyn Fn(&iced_winit::winit::window::Window)>>,
) -> iced::Result {
    let renderer_settings = renderer::Settings {
        default_font: settings.default_font,
        default_text_size: settings.default_text_size,
        text_multithreading: settings.text_multithreading,
        antialiasing: if settings.antialiasing {
            Some(renderer::settings::Antialiasing::MSAAx4)
        } else {
            None
        },
        ..renderer::Settings::from_env()
    };

    Ok(iced_run::run::<
        Instance<S>,
        S::Executor,
        renderer::window::Compositor,
        dyn Fn(&_),
    >(
        settings.into(), renderer_settings, after_window_created
    )?)
}

pub fn load_font(name: String) -> Option<Arc<Vec<u8>>> {
    let handle = SystemSource::new()
        .select_best_match(&[FamilyName::Title(name)], &Properties::default())
        .ok()?;
    
    let font = handle.load().ok()?;

    font.copy_font_data()
}
