use crate::thread_safe::ThreadSafe;
use crate::window_manager::WindowManager;

pub type ThreadSafeWindowManager = ThreadSafe<WindowManager>;
pub type ThreadSafeWindowManagers = ThreadSafe<Vec<ThreadSafeWindowManager>>;
