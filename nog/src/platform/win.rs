use std::ffi::c_void;
use std::mem;

use super::{NativeWindow, Rect, WindowId, WindowPosition, WindowSize};
use winapi::Windows::Win32::Foundation::{HWND, LPARAM, PWSTR, RECT, WPARAM};
use winapi::Windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use winapi::Windows::Win32::UI::{
    KeyboardAndMouseInput::keybd_event,
    WindowsAndMessaging::{
        GetWindowRect, GetWindowTextLengthW, GetWindowTextW, PostMessageW, SetForegroundWindow,
        SetWindowPos, SWP_NOMOVE, SWP_NOSIZE, WM_CLOSE,
    },
};

#[derive(Debug, Clone)]
pub struct Window(HWND);

impl From<HWND> for WindowId {
    fn from(hwnd: HWND) -> Self {
        Self(hwnd.0 as usize)
    }
}

impl From<WindowId> for HWND {
    fn from(hwnd: WindowId) -> Self {
        Self(hwnd.0 as isize)
    }
}

impl From<RECT> for WindowSize {
    fn from(rect: RECT) -> Self {
        Self {
            width: (rect.right - rect.left) as usize,
            height: (rect.bottom - rect.top) as usize,
        }
    }
}

impl From<RECT> for Rect {
    fn from(rect: RECT) -> Self {
        Self {
            left: rect.left as isize,
            right: rect.right as isize,
            top: rect.top as isize,
            bottom: rect.bottom as isize,
        }
    }
}

impl Window {
    pub fn from_hwnd(hwnd: HWND) -> Self {
        Self(hwnd)
    }
}

const HWND_NOTOPMOST: isize = -2;

impl Window {
    /// This function returns the size of the window INCLUDING the extend window frame
    pub fn get_full_size(&self) -> WindowSize {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect);

            WindowSize::from(rect)
        }
    }

    pub fn get_extended_window_frame(&self) -> Rect {
        unsafe {
            let mut full_rect = RECT::default();
            GetWindowRect(self.0, &mut full_rect);

            let mut win_rect = RECT::default();
            DwmGetWindowAttribute(
                self.0,
                DWMWA_EXTENDED_FRAME_BOUNDS.0 as u32,
                &mut win_rect as *mut RECT as *mut c_void,
                mem::size_of::<RECT>() as u32,
            );

            Rect::from(full_rect) - Rect::from(win_rect)
        }
    }

    /// This function returns the size of the window EXCLUDING the extend window frame
    pub fn get_window_size(&self) -> WindowSize {
        unsafe {
            let mut rect = RECT::default();
            DwmGetWindowAttribute(
                self.0,
                DWMWA_EXTENDED_FRAME_BOUNDS.0 as u32,
                &mut rect as *mut RECT as *mut c_void,
                mem::size_of::<RECT>() as u32,
            );
            WindowSize::from(rect)
        }
    }
}

impl NativeWindow for Window {
    fn new(id: WindowId) -> Self {
        Self(id.into())
    }

    fn focus(&self) {
        unsafe {
            keybd_event(0, 0, Default::default(), 0);
            SetForegroundWindow(self.0);
        }
    }

    fn close(&self) {
        unsafe {
            PostMessageW(self.0, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    fn resize(&self, mut size: WindowSize) {
        let frame_rect = self.get_extended_window_frame();
        size.width = (size.width as isize + frame_rect.right - frame_rect.left) as usize;
        size.height = (size.height as isize + frame_rect.bottom - frame_rect.top) as usize;

        unsafe {
            SetWindowPos(
                self.0,
                HWND(HWND_NOTOPMOST),
                0,
                0,
                size.width as i32,
                size.height as i32,
                SWP_NOMOVE,
            );
        }
    }

    fn reposition(&self, mut pos: WindowPosition) {
        let frame_rect = self.get_extended_window_frame();
        pos.x += frame_rect.left;
        pos.y += frame_rect.top;
        unsafe {
            SetWindowPos(
                self.0,
                HWND(HWND_NOTOPMOST),
                pos.x as i32,
                pos.y as i32,
                0,
                0,
                SWP_NOSIZE,
            );
        }
    }

    fn get_id(&self) -> WindowId {
        self.0.into()
    }

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

    fn get_size(&self) -> WindowSize {
        self.get_window_size()
    }
}
