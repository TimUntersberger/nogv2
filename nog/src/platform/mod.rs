pub trait NativeWindow: Clone + std::fmt::Debug {
    fn get_title(&self) -> String;
}

pub mod win;
pub use win::*;
