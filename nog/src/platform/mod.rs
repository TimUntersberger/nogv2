pub mod win;
use std::ops;

pub use win::*;

use crate::{display::Display, key::Key, modifiers::Modifiers};

pub trait NativeWindow: Clone + Copy + std::fmt::Debug {
    fn new(id: WindowId) -> Self;
    fn reposition(&self, pos: Position);
    fn resize(&self, size: Size);
    fn focus(&self);
    fn exists(&self) -> bool;
    fn close(&self);
    fn minimize(&self);
    fn maximize(&self);
    fn unminimize(&self);
    fn show(&self);
    fn hide(&self);
    fn remove_decorations(&self) -> Box<dyn Fn() + 'static + Send + Sync>;
    fn get_id(&self) -> WindowId;
    fn get_title(&self) -> String;
    fn get_size(&self) -> Size;
    fn get_position(&self) -> Position;
}

pub trait NativeMonitor {
    fn get_id(&self) -> MonitorId;
    fn get_work_area(&self) -> Area;
    // fn get_name() -> String;
}

pub trait NativeApi {
    type Window: NativeWindow;
    type Monitor: NativeMonitor;

    /// This function simulates keys presses
    fn simulate_key_press(key: Key, modifiers: Modifiers);
    fn launch(path: String);
    fn get_foreground_window() -> Self::Window;
    fn get_displays() -> Vec<Display>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorId(pub isize);

impl std::fmt::Display for MonitorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct WindowId(pub usize);

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub size: Size,
    pub pos: Position,
}

impl Area {
    pub fn new(size: Size, pos: Position) -> Self {
        Self { size, pos }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Size {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl ops::Sub for Size {
    type Output = Size;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: isize,
    pub right: isize,
    pub top: isize,
    pub bottom: isize,
}

impl Rect {
    pub fn as_size(self) -> Size {
        Size::new(
            (self.right - self.left).abs() as usize,
            (self.top - self.bottom).abs() as usize,
        )
    }
}

impl ops::Sub for Rect {
    type Output = Rect;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            left: self.left - rhs.left,
            right: self.right - rhs.right,
            top: self.top - rhs.top,
            bottom: self.bottom - rhs.bottom,
        }
    }
}
