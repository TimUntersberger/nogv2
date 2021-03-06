use windows::Windows::Win32::{
    Foundation::{HINSTANCE, HWND},
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetWindowLongA, TranslateMessage, EVENT_MAX, EVENT_MIN,
        EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND,
        EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART, MSG, WINDOW_LONG_PTR_INDEX,
    },
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{GetWindowTextLengthW, PeekMessageW, PM_REMOVE},
    },
};

use super::WindowEventLoop;
use crate::event::Event;
use crate::platform::Window;
use crate::{
    window_event_loop::{WindowEvent, WindowEventKind},
    EventLoop,
};
use lazy_static::lazy_static;
use log::error;
use std::{
    mem,
    sync::{
        atomic::{self, AtomicBool},
        mpsc::{sync_channel, Receiver, Sender, SyncSender},
        Arc, Mutex,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct WinApiWindowEvent {
    kind: WinApiWindowEventKind,
    hwnd: HWND,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WinApiWindowEventKind {
    Destroy,
    Hide,
    Minimize,
    Unminimize,
    Show,
    FocusChange,
}

impl WinApiWindowEventKind {
    pub fn from_u32(v: u32) -> Option<Self> {
        if v == EVENT_OBJECT_DESTROY {
            Some(Self::Destroy)
        } else if v == EVENT_OBJECT_SHOW {
            Some(Self::Show)
        } else if v == EVENT_SYSTEM_FOREGROUND {
            Some(Self::FocusChange)
        } else if v == EVENT_OBJECT_HIDE {
            Some(Self::Hide)
        } else if v == EVENT_SYSTEM_MINIMIZESTART {
            Some(Self::Minimize)
        } else if v == EVENT_SYSTEM_MINIMIZEEND {
            Some(Self::Unminimize)
        } else {
            None
        }
    }
}

lazy_static! {
    static ref CHAN: (
        SyncSender<WinApiWindowEvent>,
        Arc<Mutex<Receiver<WinApiWindowEvent>>>
    ) = {
        let (tx, rx) = sync_channel(100);

        (tx, Arc::new(Mutex::new(rx)))
    };
    static ref HOOK: Arc<Mutex<HWINEVENTHOOK>> = Arc::new(Mutex::new(HWINEVENTHOOK::default()));
    static ref STOP: AtomicBool = AtomicBool::new(false);
}

impl EventLoop for WindowEventLoop {
    fn run(tx: SyncSender<Event>) {
        STOP.store(false, atomic::Ordering::SeqCst);

        std::thread::spawn(|| unsafe {
            let hook = SetWinEventHook(
                EVENT_MIN,
                EVENT_MAX,
                HINSTANCE::NULL,
                Some(win_event_hook),
                0,
                0,
                0,
            );

            *HOOK.lock().unwrap() = hook;

            let mut msg = MSG::default();

            while !STOP.load(atomic::Ordering::SeqCst) {
                while PeekMessageW(&mut msg, HWND::NULL, 0, 0, PM_REMOVE).into() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        });

        while let Ok(event) = CHAN.1.lock().unwrap().recv() {
            let window = Window::from_hwnd(event.hwnd);
            let kind = match event.kind {
                WinApiWindowEventKind::Show => Some(WindowEventKind::Created),
                WinApiWindowEventKind::Destroy => Some(WindowEventKind::Deleted),
                WinApiWindowEventKind::FocusChange => Some(WindowEventKind::FocusChanged),
                WinApiWindowEventKind::Minimize | WinApiWindowEventKind::Hide => {
                    Some(WindowEventKind::Minimized)
                }
                _ => None,
            };

            if let Some(kind) = kind {
                tx.send(Event::Window(WindowEvent { kind, window }))
                    .unwrap();
            } else {
                // warn!("The following event is not supported: {:#?}", event)
            }
        }
    }

    fn stop() {
        unsafe {
            UnhookWinEvent(mem::take(&mut *HOOK.lock().unwrap()));
        }
        STOP.store(true, atomic::Ordering::SeqCst)
    }
}

const OBJID_WINDOW: i32 = 0;
const GWL_STYLE: WINDOW_LONG_PTR_INDEX = WINDOW_LONG_PTR_INDEX(-16);
const WS_CHILD: u32 = 0x40000000;
const WS_POPUP: u32 = 0x80000000;

unsafe extern "system" fn win_event_hook(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    idobject: i32,
    _idchild: i32,
    _ideventthread: u32,
    _dwmseventtime: u32,
) {
    if idobject != OBJID_WINDOW {
        return;
    }

    let style = GetWindowLongA(hwnd, GWL_STYLE) as u32;
    let is_child_window = (style & WS_CHILD) == WS_CHILD;
    let is_popup_window = (style & WS_POPUP) == WS_POPUP;

    if is_child_window || is_popup_window {
        return;
    }

    // A bit hacky, but windows without a title are usually irrelevant. At least I haven't found a
    // window yet where this isn't the case.
    if GetWindowTextLengthW(hwnd) == 0 {
        return;
    }

    if let Some(kind) = WinApiWindowEventKind::from_u32(event) {
        let event = WinApiWindowEvent { kind, hwnd };

        if let Err(e) = CHAN.0.send(event) {
            error!("{}", e);
        }
    }
}
