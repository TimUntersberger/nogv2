use std::path::PathBuf;

pub fn get_bin_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        let mut path: PathBuf = std::env::current_exe().unwrap();
        path.pop();
        path.pop();
        path.pop();
        path.push("target");
        path.push("debug");
        path
    }
    #[cfg(not(debug_assertions))]
    {
        let mut path: PathBuf = dirs::data_dir().unwrap_or_default();
        path.push("nog");
        path.push("bin");
        path
    }
}

pub fn get_runtime_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        let mut path: PathBuf = std::env::current_exe().unwrap();
        path.pop();
        path.pop();
        path.pop();
        path.push("nog");
        path.push("runtime");
        path
    }
    #[cfg(not(debug_assertions))]
    {
        let mut path: PathBuf = dirs::data_dir().unwrap_or_default();
        path.push("nog");
        path.push("runtime");
        path
    }
}

pub fn get_config_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        let mut path: PathBuf = std::env::current_exe().unwrap();
        path.pop();
        path.pop();
        path.pop();
        path.push("nog");
        path.push("config");
        path
    }
    #[cfg(not(debug_assertions))]
    {
        let mut path: PathBuf = dirs::config_dir().unwrap_or_default();
        path.push("nog");
        path
    }
}
