use winapi::Windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use winapi::Windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, PeekMessageW, SetWindowsHookExA, TranslateMessage,
    UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, PM_REMOVE, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::event::Event;
use crate::keybinding_event_loop::Modifiers;
use crate::EventLoop;
use lazy_static::lazy_static;
use log::{debug, info};
use std::mem;
use std::sync::atomic;
use std::sync::{
    atomic::AtomicBool,
    mpsc::{sync_channel, Receiver, SyncSender},
    Arc, Mutex, RwLock,
};

use super::{InputEvent, Keybinding, KeybindingEventLoop};

lazy_static! {
    static ref CHAN: (SyncSender<InputEvent>, Arc<Mutex<Receiver<InputEvent>>>) = {
        let (tx, rx) = sync_channel(100);

        (tx, Arc::new(Mutex::new(rx)))
    };
    static ref HOOK: Arc<Mutex<HHOOK>> = Arc::new(Mutex::new(HHOOK::default()));
    static ref MODIFIERS: Arc<Mutex<Modifiers>> = Arc::new(Mutex::new(Modifiers::default()));
    static ref KEYBINDING_IDS: Arc<RwLock<Vec<usize>>> = Arc::new(RwLock::new(vec![]));
    static ref STOP: AtomicBool = AtomicBool::new(false);
}

impl KeybindingEventLoop {
    pub fn update_keybindings(keybindings: Vec<Keybinding>) {
        *KEYBINDING_IDS.write().unwrap() = keybindings.iter().map(|kb| kb.get_id()).collect();
        info!("Keybinding event loop received new keybindings");
    }
}

impl EventLoop for KeybindingEventLoop {
    fn run(tx: std::sync::mpsc::Sender<Event>) {
        STOP.store(false, atomic::Ordering::SeqCst);

        std::thread::spawn(|| unsafe {
            let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_hook), HINSTANCE::NULL, 0);

            *HOOK.lock().unwrap() = hook;

            let mut msg = MSG::default();

            while !STOP.load(atomic::Ordering::SeqCst) {
                while PeekMessageW(&mut msg, HWND::NULL, 0, 0, PM_REMOVE).into() {
                    TranslateMessage(&mut msg);
                    DispatchMessageW(&mut msg);
                }
            }
        });

        while let Ok(event) = CHAN.1.lock().unwrap().recv() {
            if let InputEvent::KeyUp(kb) = event {
                tx.send(Event::Keybinding(kb)).unwrap();
            }
        }
    }

    fn stop() {
        unsafe {
            UnhookWindowsHookEx(mem::take(&mut *HOOK.lock().unwrap()));
        }
        STOP.store(true, atomic::Ordering::SeqCst)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum KbdHookEvent {
    KeyDown,
    KeyUp,
    SysKeyUp,
    SysKeyDown,
}

impl KbdHookEvent {
    pub fn from_usize(input: usize) -> Option<Self> {
        match input as u32 {
            WM_KEYDOWN => Some(Self::KeyDown),
            WM_KEYUP => Some(Self::KeyUp),
            WM_SYSKEYUP => Some(Self::SysKeyUp),
            WM_SYSKEYDOWN => Some(Self::SysKeyDown),
            _ => None,
        }
    }
}

unsafe extern "system" fn keyboard_hook(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode >= 0 {
        let kbdhook = lparam.0 as *mut KBDLLHOOKSTRUCT;
        let key = (*kbdhook).vkCode;
        let kbd_event = KbdHookEvent::from_usize(wparam.0).unwrap();
        let mut event = None;

        match kbd_event {
            KbdHookEvent::KeyDown | KbdHookEvent::SysKeyDown => {
                match key {
                    162 => MODIFIERS.lock().unwrap().ctrl = true,
                    160 | 161 => MODIFIERS.lock().unwrap().shift = true,
                    91 => MODIFIERS.lock().unwrap().win = true,
                    164 => MODIFIERS.lock().unwrap().lalt = true,
                    165 => MODIFIERS.lock().unwrap().ralt = true,
                    key => {
                        event = Some(InputEvent::KeyDown(Keybinding {
                            key_code: key as usize,
                            modifiers: MODIFIERS.lock().unwrap().clone(),
                        }));
                    }
                };
            }
            KbdHookEvent::KeyUp | KbdHookEvent::SysKeyUp => {
                match key {
                    162 => MODIFIERS.lock().unwrap().ctrl = false,
                    160 | 161 => MODIFIERS.lock().unwrap().shift = false,
                    91 => MODIFIERS.lock().unwrap().win = false,
                    164 => MODIFIERS.lock().unwrap().lalt = false,
                    165 => MODIFIERS.lock().unwrap().ralt = false,
                    key => {
                        event = Some(InputEvent::KeyUp(Keybinding {
                            key_code: key as usize,
                            modifiers: MODIFIERS.lock().unwrap().clone(),
                        }));
                    }
                };
            }
        }

        if let Some(event) = event {
            match event {
                InputEvent::KeyUp(kb) | InputEvent::KeyDown(kb) => {
                    let ev_id = kb.get_id();
                    if KEYBINDING_IDS
                        .read()
                        .unwrap()
                        .iter()
                        .find(|id| *id == &ev_id)
                        .is_some()
                    {
                        debug!("blocking {:#?}", event);
                        CHAN.0.send(event).unwrap();
                        return LRESULT(1);
                    }
                }
                _ => {}
            }
        }
    }

    CallNextHookEx(HHOOK::NULL, ncode, wparam, lparam)
}
