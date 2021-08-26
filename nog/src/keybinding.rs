use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum KeybindingMode {
    Global,
    Workspace,
    Normal
}

impl FromStr for KeybindingMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "g" => KeybindingMode::Global,
            "w" => KeybindingMode::Workspace,
            "n" => KeybindingMode::Normal,
            _ => return Err(())
        })
    }
}
