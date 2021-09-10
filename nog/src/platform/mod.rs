pub trait NativeWindow: Clone + Copy + std::fmt::Debug {
    fn new(id: WindowId) -> Self;
    fn reposition(&self, pos: Position);
    fn resize(&self, size: Size);
    fn focus(&self);
    fn exists(&self) -> bool;
    fn close(&self);
    fn remove_decorations(&self) -> Box<dyn Fn() -> () + 'static>;
    fn get_id(&self) -> WindowId;
    fn get_title(&self) -> String;
    fn get_size(&self) -> Size;
    fn get_position(&self) -> Position;
}

pub trait NativeDisplay {
    fn get_id() -> String;
    fn get_size(&self, config: &Config) -> Size;
    fn hide_taskbar(&self);
    fn show_taskbar(&self);
    // fn get_name() -> String;
}

pub trait NativeApi {
    type Window: NativeWindow;
    type Display: NativeDisplay;

    fn get_foreground_window() -> Self::Window;
    fn get_displays() -> Vec<Self::Display>;
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct WindowId(pub usize);

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

pub mod win;
use std::ops;

pub use win::*;

use crate::config::Config;
