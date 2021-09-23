use std::{mem, ptr};

use widestring::WideCString;
use windows::Windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, PWSTR},
    Graphics::Gdi::{
        EnumDisplayDevicesW, MonitorFromWindow, DISPLAY_DEVICEW,
        DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, MONITOR_DEFAULTTONEAREST,
    },
    UI::WindowsAndMessaging::{EnumWindows, GetForegroundWindow},
};

use crate::{
    display::{Display, DisplayId},
    platform::{Monitor, NativeApi, Window},
    window_manager::WindowManager,
};

unsafe extern "system" fn enum_windows_task_bars_cb(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let taskbars = &mut *(l_param.0 as *mut Vec<HWND>);

    match Window::from_hwnd(hwnd).get_class_name().as_str() {
        "Shell_TrayWnd" | "Shell_SecondaryTrayWnd" => {
            taskbars.push(hwnd);
        }
        _ => {}
    };

    true.into()
}

#[derive(Debug)]
struct DisplayDevice {
    pub name: String,
    pub string: String,
    pub id: String,
}

pub struct Api;

impl Api {
    fn get_display_devices() -> Vec<DisplayDevice> {
        let mut display_devices = Vec::new();

        unsafe {
            let mut display_device = DISPLAY_DEVICEW::default();
            display_device.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;

            let mut idx = 0;
            while EnumDisplayDevicesW(PWSTR(ptr::null_mut()), idx, &mut display_device, 0).into() {
                let is_attached = (display_device.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP)
                    == DISPLAY_DEVICE_ATTACHED_TO_DESKTOP;

                // We only care about displays that are actually getting used by the desktop
                // environment
                if is_attached {
                    let device_name = WideCString::from_ptr_str(display_device.DeviceName.as_ptr())
                        .to_string_lossy();

                    let device_string =
                        WideCString::from_ptr_str(display_device.DeviceString.as_ptr())
                            .to_string_lossy();

                    let device_id = WideCString::from_ptr_str(display_device.DeviceID.as_ptr())
                        .to_string_lossy();

                    display_devices.push(DisplayDevice {
                        name: device_name,
                        string: device_string,
                        id: device_id,
                    });
                }

                idx += 1;
            }
        }

        display_devices
    }

    fn get_taskbar_windows() -> Vec<Window> {
        let mut taskbars: Vec<HWND> = Vec::new();

        unsafe {
            //EnumWindows is synchronous
            EnumWindows(
                Some(enum_windows_task_bars_cb),
                LPARAM(&mut taskbars as *mut Vec<HWND> as isize),
            );
        }

        taskbars.into_iter().map(Window::from_hwnd).collect()
    }
}

impl NativeApi for Api {
    type Window = Window;
    type Monitor = Monitor;

    fn get_foreground_window() -> Self::Window {
        unsafe { Window::from_hwnd(GetForegroundWindow()) }
    }

    fn get_displays() -> Vec<Display> {
        let devices = Self::get_display_devices();
        assert!(!devices.is_empty(), "Somehow not a single display device was found");

        let taskbars = Self::get_taskbar_windows();
        assert!(!devices.is_empty(), "Somehow not a taskbar was found");

        taskbars
            .into_iter()
            .map(|tb| unsafe {
                let hmonitor = MonitorFromWindow(tb.get_hwnd(), MONITOR_DEFAULTTONEAREST);
                let monitor = Monitor::from_hmonitor(hmonitor);

                let id = DisplayId(devices.iter().find(|dev| dev.name == monitor.device_name).expect("Devices and monitors don't match. Something must have went wrong during initialization").id.clone());

                Display {
                    id,
                    taskbar_win: tb,
                    bar: None,
                    wm: WindowManager::new(),
                    monitor,
                }
            })
            .collect()
    }
}
