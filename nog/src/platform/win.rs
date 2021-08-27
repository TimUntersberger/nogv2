use super::NativeWindow;
use winapi::Windows::Win32::Foundation::{HWND, PWSTR, RECT};
use winapi::Windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW, GetWindowRect};

#[derive(Debug, Clone)]
pub struct Window(pub HWND);

impl NativeWindow for Window {
    fn get_title(&self) -> String {
        unsafe {
            // GetWindowTextLengthW returns the length of the title without the null character,
            // which means we have to add one the length to the get the correct buffer size.
            let len = GetWindowTextLengthW(self.0) + 1;
            let mut buffer = vec![0u16; len as usize];
            GetWindowTextW(self.0, PWSTR(buffer.as_mut_ptr()), len);

            String::from_utf16(buffer.as_slice()).unwrap()
        }
    }

    fn get_size(&self) -> (usize, usize) {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect);

            ((rect.right - rect.left) as usize, (rect.bottom - rect.top) as usize)
        }
    }
}
