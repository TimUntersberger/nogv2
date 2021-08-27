pub trait NativeWindow: Clone + std::fmt::Debug {
    fn get_title(&self) -> String;
    fn get_size(&self) -> (usize, usize);
}

pub mod win;
pub use win::*;
