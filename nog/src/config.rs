use crate::rgb::Rgb;

pub struct Config {
    pub color: Rgb,
    pub bar_height: u32,
    pub font_size: u32,
    pub font_name: String,
    pub use_border: bool,
    pub enable_hot_reloading: bool,
    /// Any window with a smaller width than this won't get managed automatically
    pub min_width: usize,
    /// Any window with a smaller height than this won't get managed automatically
    pub min_height: usize,
    /// This only changes the way the different colors get derived from the color provied by the
    /// user and whether to use a black or white text color.
    pub light_theme: bool,
    pub multi_monitor: bool,
    pub launch_on_startup: bool,
    pub outer_gap: u32,
    pub inner_gap: u32,
    pub remove_decorations: bool,
    pub remove_task_bar: bool,
    /// When enabled nog won't respond to the following actions when a window is fullscreened:
    ///     * swap
    ///     * focus
    pub ignore_fullscreen_actions: bool,
    pub display_app_bar: bool,
    // not needed because the user can just use Alt if he wants both and either LAlt or RAlt if he
    // only wants one.
    // pub allow_alt_right: bool
}

impl Config {
    pub fn get_text_color(&self) -> Rgb {
        if self.light_theme {
            Rgb::BLACK
        } else {
            Rgb::WHITE
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            color: Rgb::from_hex(0x3f3f3f),
            bar_height: 24,
            font_size: 18,
            font_name: "Consolas".into(),
            use_border: false,
            enable_hot_reloading: true,
            min_width: 200,
            min_height: 200,
            light_theme: false,
            multi_monitor: false,
            launch_on_startup: true,
            outer_gap: 0,
            inner_gap: 0,
            remove_decorations: true,
            remove_task_bar: true,
            ignore_fullscreen_actions: false,
            display_app_bar: true,
        }
    }
}
