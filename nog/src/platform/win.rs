use std::ffi::c_void;
use std::{mem, ptr};

use super::{
    Area, MonitorId, NativeApi, NativeMonitor, NativeWindow, Position, Rect, Size, WindowId,
};
use winapi::Windows::Win32::Foundation::{HWND, LPARAM, PWSTR, RECT, WPARAM};
use winapi::Windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use winapi::Windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO};
use winapi::Windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowLongW, SetWindowLongW, ShowWindow, GWL_EXSTYLE, GWL_STYLE,
    SW_HIDE, SW_SHOW, WS_CAPTION, WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME, WS_EX_STATICEDGE,
    WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
};
use winapi::Windows::Win32::UI::{
    KeyboardAndMouseInput::keybd_event,
    WindowsAndMessaging::{
        FindWindowW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsWindow, PostMessageW,
        SetForegroundWindow, SetWindowPos, SWP_NOMOVE, SWP_NOSIZE, WM_CLOSE,
    },
};

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

impl From<RECT> for Size {
    fn from(rect: RECT) -> Self {
        Self {
            width: (rect.right - rect.left) as usize,
            height: (rect.bottom - rect.top) as usize,
        }
    }
}

impl From<RECT> for Area {
    fn from(rect: RECT) -> Self {
        Self {
            size: Size::from(rect),
            pos: Position::from(rect),
        }
    }
}

impl From<RECT> for Position {
    fn from(rect: RECT) -> Self {
        Self {
            x: rect.left as isize,
            y: rect.top as isize,
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
    pub fn get_full_size(&self) -> Size {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect);

            Size::from(rect)
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
            )
            .unwrap();

            Rect::from(full_rect) - Rect::from(win_rect)
        }
    }

    /// This function returns the size of the window EXCLUDING the extend window frame
    pub fn get_window_size(&self) -> Size {
        unsafe {
            let mut rect = RECT::default();
            DwmGetWindowAttribute(
                self.0,
                DWMWA_EXTENDED_FRAME_BOUNDS.0 as u32,
                &mut rect as *mut RECT as *mut c_void,
                mem::size_of::<RECT>() as u32,
            )
            .unwrap();
            Size::from(rect)
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

    fn exists(&self) -> bool {
        unsafe { IsWindow(self.0).into() }
    }

    fn remove_decorations(&self) -> Box<dyn Fn() + 'static> {
        unsafe {
            let style = GetWindowLongW(self.0, GWL_STYLE) as u32;
            let new_style = style
                & !(WS_CAPTION.0
                    | WS_THICKFRAME.0
                    | WS_MINIMIZEBOX.0
                    | WS_MAXIMIZEBOX.0
                    | WS_SYSMENU.0);
            SetWindowLongW(self.0, GWL_STYLE, new_style as i32);

            let exstyle = GetWindowLongW(self.0, GWL_EXSTYLE) as u32;
            let new_exstyle =
                exstyle & !(WS_EX_DLGMODALFRAME.0 | WS_EX_CLIENTEDGE.0 | WS_EX_STATICEDGE.0);
            SetWindowLongW(self.0, GWL_EXSTYLE, new_exstyle as i32);

            let hwnd = self.0;
            Box::new(move || {
                SetWindowLongW(hwnd, GWL_STYLE, style as i32);
                SetWindowLongW(hwnd, GWL_EXSTYLE, exstyle as i32);
            })
        }
    }

    fn resize(&self, mut size: Size) {
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

    fn reposition(&self, mut pos: Position) {
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

    fn get_size(&self) -> Size {
        self.get_window_size()
    }

    fn get_position(&self) -> Position {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect);

            Position::from(rect)
        }
    }
}

pub struct Api;

impl NativeApi for Api {
    type Window = Window;
    type Monitor = Monitor;

    fn get_foreground_window() -> Self::Window {
        unsafe { Window::from_hwnd(GetForegroundWindow()) }
    }

    fn get_displays() -> Vec<Self::Monitor> {
        vec![]
    }
}

pub struct Monitor {
    id: MonitorId,
    primary: bool,
    taskbar_hwnd: HWND,
}

impl Monitor {
    pub fn new(primary: bool, hmonitor: HMONITOR) -> Self {
        unsafe {
            let taskbar_hwnd = FindWindowW(
                PWSTR(
                    "Shell_TrayWnd"
                        .encode_utf16()
                        .collect::<Vec<_>>()
                        .as_mut_ptr(),
                ),
                PWSTR(ptr::null_mut()),
            );

            assert!(
                taskbar_hwnd.0 != 0,
                "The taskbar of some display couldn't be found!"
            );

            let mut monitor_info = MONITORINFO::default();
            GetMonitorInfoW(hmonitor, &mut monitor_info as *mut MONITORINFO);

            Self {
                id: MonitorId(hmonitor.0),
                primary,
                taskbar_hwnd,
            }
        }
    }
}

impl NativeMonitor for Monitor {
    fn get_id(&self) -> MonitorId {
        self.id
    }

    fn get_work_area(&self) -> Area {
        let hmonitor = HMONITOR(self.id.0);
        let mut monitor_info = MONITORINFO::default();

        unsafe {
            GetMonitorInfoW(hmonitor, &mut monitor_info as *mut MONITORINFO);
        }

        Area::from(monitor_info.rcWork)
    }

    //     fn get_pos(&self, config: &Config) -> Position {
    //         let mut pos = Position::new(0, 0);

    //         if config.display_app_bar {
    //             pos.y += config.bar_height as isize;
    //         }

    //         pos
    //     }

    //     fn get_size(&self, config: &Config) -> Size {
    //         let win = Window::from_hwnd(self.taskbar_hwnd);
    //         let mut size = Size::new(1920, 1080);

    //         if config.remove_task_bar {
    //             size.height -= win.get_size().height;
    //         }

    //         if config.display_app_bar {
    //             size.height -= config.bar_height as usize;
    //         }

    //         size
    //     }

    fn hide_taskbar(&self) {
        unsafe {
            ShowWindow(self.taskbar_hwnd, SW_HIDE);
        }
    }

    fn show_taskbar(&self) {
        unsafe {
            ShowWindow(self.taskbar_hwnd, SW_SHOW);
        }
    }
}
