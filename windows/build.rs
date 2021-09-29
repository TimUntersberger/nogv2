fn main() {
    windows::build!(
        Windows::Win32::UI::WindowsAndMessaging::*,
        Windows::Win32::UI::KeyboardAndMouseInput::*,
        Windows::Win32::System::ApplicationInstallationAndServicing::*,
        Windows::Win32::Foundation::*,
        Windows::Win32::Graphics::Dwm::*,
        Windows::Win32::Graphics::Gdi::*,
        Windows::Win32::UI::Accessibility::*,
        Windows::Win32::UI::Shell::*,
        Windows::Win32::System::Diagnostics::Debug::*,
        Windows::Win32::Storage::StructuredStorage::*,
        Windows::Win32::System::LibraryLoader::*,
        Windows::Win32::System::SystemServices::*,
        Windows::Win32::System::Memory::*,
        Windows::Win32::System::Com::*
    );
}
