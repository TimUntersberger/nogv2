use rgb::Rgb;

pub struct Config {
    pub color: Rgb,
    pub bar_height: u32,
    pub font_size: u32,
    pub font_name: String,
    /// This only changes the way the different colors get derived from the color provied by the
    /// user and whether to use a black or white text color.
    pub light_theme: bool,
    pub multi_monitor: bool,
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
            light_theme: false,
            multi_monitor: false,
            outer_gap: 0,
            inner_gap: 0,
            remove_decorations: true,
            remove_task_bar: true,
            ignore_fullscreen_actions: false,
            display_app_bar: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigProperty {
    Color(Rgb),
    BarHeight(u32),
    FontSize(u32),
    FontName(String),
    LightTheme(bool),
    MultiMonitor(bool),
    OuterGap(u32),
    InnerGap(u32),
    RemoveDecorations(bool),
    RemoveTaskBar(bool),
    IgnoreFullscreenActions(bool),
    DisplayAppBar(bool),
}

impl ConfigProperty {
    pub fn get_name(&self) -> &'static str {
        match self {
            ConfigProperty::Color(_) => "color",
            ConfigProperty::BarHeight(_) => "bar_height",
            ConfigProperty::FontSize(_) => "font_size",
            ConfigProperty::FontName(_) => "font_name",
            ConfigProperty::LightTheme(_) => "light_theme",
            ConfigProperty::MultiMonitor(_) => "mulit_monitor",
            ConfigProperty::OuterGap(_) => "outer_gap",
            ConfigProperty::InnerGap(_) => "inner_gap",
            ConfigProperty::RemoveDecorations(_) => "remove_decorations",
            ConfigProperty::RemoveTaskBar(_) => "remove_task_bar",
            ConfigProperty::IgnoreFullscreenActions(_) => "ignore_fullscreen_actions",
            ConfigProperty::DisplayAppBar(_) => "display_app_bar",
        }
    }
}
