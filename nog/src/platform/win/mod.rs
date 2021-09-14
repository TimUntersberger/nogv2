use winapi::Windows::Win32::Foundation::RECT;

use crate::platform::{Area, Position, Rect, Size};

pub mod api;
pub mod monitor;
pub mod window;

pub use api::*;
pub use monitor::*;
pub use window::*;

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
