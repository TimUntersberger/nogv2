fn main() {
    if cfg!(target_os = "windows") {
        winres::WindowsResource::new()
            .set_icon_with_id("../assets/logo.ico", "logo.ico")
            .compile()
            .unwrap();
    }
}
