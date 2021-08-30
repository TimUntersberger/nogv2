pub trait NativeWindow: Clone + Copy + std::fmt::Debug {
    fn new(id: WindowId) -> Self;
    fn get_foreground_window() -> Self;
    fn reposition(&self, pos: WindowPosition);
    fn resize(&self, size: WindowSize);
    fn focus(&self);
    fn exists(&self) -> bool;
    fn close(&self);
    fn remove_decorations(&self) -> Box<dyn Fn() -> () + 'static>;
    fn get_id(&self) -> WindowId;
    fn get_title(&self) -> String;
    fn get_size(&self) -> WindowSize;
    fn get_position(&self) -> WindowPosition;
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct WindowId(pub usize);

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowSize {
    pub width: usize,
    pub height: usize,
}

impl WindowSize {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl ops::Sub for WindowSize {
    type Output = WindowSize;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPosition {
    pub x: isize,
    pub y: isize,
}

impl WindowPosition {
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
