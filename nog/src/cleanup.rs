#[derive(Default)]
pub struct WindowCleanup {
    pub add_decorations: Option<Box<dyn Fn() -> ()>>,
    /// Resets the position and size. We combine both cleanups, since there is no case where we
    /// only change the size or position and not the other one.
    pub reset_transform: Option<Box<dyn Fn() -> ()>>,
}
