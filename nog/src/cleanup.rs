#[derive(Default)]
pub struct WindowCleanup {
    pub add_decorations: Option<Box<dyn Fn()>>,
    /// Resets the position and size. We combine both cleanups, since there is no case where we
    /// only change the size or position and not the other one.
    pub reset_transform: Option<Box<dyn Fn()>>,
}

impl std::fmt::Debug for WindowCleanup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowCleanup")
            .field("add_decorations", &self.add_decorations.is_some())
            .field("reset_transform", &self.reset_transform.is_some())
            .finish()
    }
}

#[derive(Default, Debug)]
pub struct WorkspaceCleanup {}

#[derive(Default)]
pub struct DisplayCleanup {
    pub show_taskbar: Option<Box<dyn Fn()>>,
}

impl std::fmt::Debug for DisplayCleanup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisplayCleanup")
            .field("show_taskbar", &self.show_taskbar.is_some())
            .finish()
    }
}
