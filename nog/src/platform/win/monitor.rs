use std::{mem, ptr};
use widestring::WideCString;
use winapi::Windows::Win32::Foundation::{HWND, PWSTR};
use winapi::Windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use winapi::Windows::Win32::UI::WindowsAndMessaging::{FindWindowW, ShowWindow, SW_HIDE, SW_SHOW};

use crate::platform::{Area, MonitorId, NativeMonitor, Position};

#[derive(Debug)]
pub struct Monitor {
    /// HMONITOR
    pub id: MonitorId,
    pub device_name: String,
    pub primary: bool
}

impl Monitor {
    pub fn from_hmonitor(hmonitor: HMONITOR) -> Self {
        unsafe {
            let mut monitor_info_ex = MONITORINFOEXW::default();
            monitor_info_ex.__AnonymousBase_winuser_L13558_C43.cbSize =
                mem::size_of::<MONITORINFOEXW>() as u32;

            GetMonitorInfoW(
                hmonitor,
                &mut monitor_info_ex as *mut MONITORINFOEXW as isize as *mut MONITORINFO,
            );

            // What the fuck is this name? haha
            let monitor_info = monitor_info_ex.__AnonymousBase_winuser_L13558_C43;

            let primary = Position::from(monitor_info.rcMonitor) == Position::new(0, 0);
            let device_name =
                WideCString::from_ptr_str(monitor_info_ex.szDevice.as_ptr()).to_string_lossy();

            Self {
                id: MonitorId(hmonitor.0),
                primary,
                device_name,
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
        monitor_info.cbSize = mem::size_of::<MONITORINFO>() as u32;

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
}
