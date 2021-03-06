use std::ffi::c_void;
use std::{mem, ptr};

use windows::Windows::Win32::Foundation::{HWND, LPARAM, PWSTR, RECT, WPARAM};
use windows::Windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetClientRect, GetSystemMetrics, GetWindowLongW, GetWindowPlacement,
    PostMessageW, SendMessageA, SendMessageW, SendNotifyMessageW, SetWindowLongW,
    SetWindowPlacement, GWL_EXSTYLE, GWL_STYLE, SC_CLOSE, SC_RESTORE, SM_CYCAPTION, SWP_DRAWFRAME,
    SWP_FRAMECHANGED, SWP_NOCOPYBITS, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SW_SHOW,
    SW_SHOWMINIMIZED, WM_SYSCOMMAND, WS_CAPTION, WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME,
    WS_EX_STATICEDGE, WS_MAXIMIZE, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
};
use windows::Windows::Win32::UI::{
    KeyboardAndMouseInput::keybd_event,
    WindowsAndMessaging::{
        GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsWindow, SetForegroundWindow,
        SetWindowPos, ShowWindow, SWP_NOMOVE, SWP_NOSIZE,
    },
};

use crate::platform::{NativeWindow, Position, Rect, Size, WindowId};
use widestring::WideCString;

#[derive(Debug, Copy, Clone)]
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

const HWND_NOTOPMOST: isize = -2;

impl Window {
    pub fn from_hwnd(hwnd: HWND) -> Self {
        Self(hwnd)
    }

    #[inline]
    pub fn get_hwnd(&self) -> HWND {
        self.0
    }

    pub fn get_class_name(&self) -> String {
        let len = 20;
        let mut buffer = vec![0u16; len];

        unsafe {
            GetClassNameW(self.0, PWSTR(buffer.as_mut_ptr()), len as i32);

            WideCString::from_ptr_str(buffer.as_ptr()).to_string_lossy()
        }
    }

    /// This function returns the size of the window INCLUDING the extend window frame
    pub fn get_full_size(&self) -> Size {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect);

            Size::from(rect)
        }
    }

    /// This function returns the rectangle of the full window
    pub fn get_window_rect(&self) -> Rect {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect);

            Rect::from(rect)
        }
    }

    /// This function returns the rectangle of the paint area of the window
    pub fn get_client_rect(&self) -> Rect {
        unsafe {
            let mut rect = RECT::default();
            GetClientRect(self.0, &mut rect);

            Rect::from(rect)
        }
    }

    pub fn get_extended_window_frame(&self) -> windows::Result<Rect> {
        unsafe {
            let mut full_rect = RECT::default();
            GetWindowRect(self.0, &mut full_rect);

            let mut win_rect = RECT::default();

            DwmGetWindowAttribute(
                self.0,
                DWMWA_EXTENDED_FRAME_BOUNDS.0 as u32,
                &mut win_rect as *mut RECT as *mut c_void,
                mem::size_of::<RECT>() as u32,
            )?;

            Ok(Rect::from(full_rect) - Rect::from(win_rect))
        }
    }

    /// This function returns the size of the window EXCLUDING the extend window frame
    pub fn get_window_size(&self) -> windows::Result<Size> {
        unsafe {
            let mut rect = RECT::default();

            DwmGetWindowAttribute(
                self.0,
                DWMWA_EXTENDED_FRAME_BOUNDS.0 as u32,
                &mut rect as *mut RECT as *mut c_void,
                mem::size_of::<RECT>() as u32,
            )?;

            Ok(Size::from(rect))
        }
    }

    pub fn is_maximized(&self) -> bool {
        unsafe {
            let mut window_placement = Default::default();
            GetWindowPlacement(self.0, &mut window_placement);
            window_placement.showCmd == SW_MAXIMIZE
        }
    }

    pub fn restore_placement(&self) {
        unsafe {
            let mut window_placement = Default::default();
            GetWindowPlacement(self.0, &mut window_placement);
            window_placement.showCmd = SW_RESTORE;
            SetWindowPlacement(self.0, &mut window_placement);
        }
    }
}

impl NativeWindow for Window {
    fn new(id: WindowId) -> Self {
        Self(id.into())
    }

    fn reposition(&self, mut pos: Position) {
        if let Ok(frame_rect) = self.get_extended_window_frame() {
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
    }

    fn resize(&self, mut size: Size) {
        if let Ok(frame_rect) = self.get_extended_window_frame() {
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
    }

    fn focus(&self) {
        unsafe {
            keybd_event(0, 0, Default::default(), 0);
            SetForegroundWindow(self.0);
        }
    }

    fn exists(&self) -> bool {
        unsafe { IsWindow(self.0).into() }
    }

    fn close(&self) {
        unsafe {
            SendNotifyMessageW(self.0, WM_SYSCOMMAND, WPARAM(SC_CLOSE as usize), LPARAM(0));
        }
    }

    fn remove_decorations(&self) -> Box<dyn Fn() + 'static + Send + Sync> {
        unsafe {
            let non_client_area_height =
                (self.get_window_rect().as_size() - self.get_client_rect().as_size()).height;
            let titlebar_height = GetSystemMetrics(SM_CYCAPTION) as usize;

            let style = GetWindowLongW(self.0, GWL_STYLE) as u32;
            let exstyle = GetWindowLongW(self.0, GWL_EXSTYLE) as u32;

            // If the non client area height is greater than the height of the titlebar, we assume
            // that the window doesn't use a custom titlebar. Sadly there is no official supported
            // way of testing for custom titlebars.
            //
            // It doesn't matter whether we remove these styles from applications with a custom
            // titlebar like discord, but firefox and chromium have buggy painting doing so.
            //
            // Thus I think just not removing these styles for applications with a custom titlebar
            // is the only real solution.
            if non_client_area_height >= titlebar_height {
                let new_style = style & !(WS_CAPTION.0 | WS_THICKFRAME.0 | WS_SYSMENU.0);
                SetWindowLongW(self.0, GWL_STYLE, new_style as i32);

                let new_exstyle =
                    exstyle & !(WS_EX_DLGMODALFRAME.0 | WS_EX_CLIENTEDGE.0 | WS_EX_STATICEDGE.0);
                SetWindowLongW(self.0, GWL_EXSTYLE, new_exstyle as i32);
            }

            let hwnd = self.0;
            Box::new(move || {
                SetWindowLongW(hwnd, GWL_STYLE, style as i32);
                SetWindowLongW(hwnd, GWL_EXSTYLE, exstyle as i32);
            })
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

            WideCString::from_ptr_str(buffer.as_ptr()).to_string_lossy()
        }
    }

    fn get_size(&self) -> Size {
        self.get_window_size().unwrap()
    }

    fn get_position(&self) -> Position {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect);

            Position::from(rect)
        }
    }

    fn show(&self) {
        unsafe {
            ShowWindow(self.0, SW_SHOW);
        }
    }

    fn hide(&self) {
        unsafe {
            ShowWindow(self.0, SW_HIDE);
        }
    }

    fn minimize(&self) {
        unsafe {
            let mut window_placement = Default::default();
            GetWindowPlacement(self.0, &mut window_placement);
            window_placement.showCmd = SW_SHOWMINIMIZED;
            SetWindowPlacement(self.0, &mut window_placement);
        }
    }

    fn maximize(&self) {
        unsafe {
            ShowWindow(self.0, SW_MAXIMIZE);
        }
    }

    fn unminimize(&self) {
        self.restore_placement();
    }
}
