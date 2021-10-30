pub fn get_version() -> &'static str {
    option_env!("NOG_VERSION").unwrap_or("DEV")
}
