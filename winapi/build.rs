fn main() {
    windows::build!(
        Windows::Win32::UI::WindowsAndMessaging::*,
        Windows::Win32::UI::KeyboardAndMouseInput::*,
        Windows::Win32::Foundation::*,
        Windows::Win32::Graphics::Dwm::*,
        Windows::Win32::Graphics::Gdi::*,
        Windows::Win32::UI::Accessibility::*
    );
}
