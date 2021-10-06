use serde::{Deserialize, Serialize};
pub use serde_json as json;

#[derive(Debug, Serialize, Deserialize)]
pub enum BarItemAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BarItem {
    pub alignment: BarItemAlignment,
    pub fg: [f32; 3],
    pub bg: [f32; 3],
    pub text: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BarContent {
    pub height: usize,
    pub font_name: String,
    pub font_size: u32,
    pub bg: [f32; 3],
    pub items: Vec<BarItem>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Window {
    pub id: usize
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub id: usize,
    pub layout: String,
    pub focused_window_id: Option<usize>,
    pub windows: Vec<Window>
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Display {
    pub id: String,
    pub monitor_id: usize,
    pub focused_workspace_id: usize,
    pub workspaces: Vec<Workspace>
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct State {
    pub focused_display_id: String,
    pub displays: Vec<Display>
}

#[derive(Debug)]
pub enum Message {
    ExecuteLua { code: String, print_type: bool },
    GetBarContent,
    GetState
}

#[derive(Debug)]
pub enum DeserializeError {
    InvalidFormat,
}

impl Message {
    pub fn serialize(&self) -> Vec<u8> {
        use Message::*;

        let mut serialized = match self {
            ExecuteLua { code, print_type } => format!("ExecuteLua:{}:{}", print_type, code),
            GetBarContent => String::from("GetBarContent:"),
            GetState => String::from("GetState:"),
        }
        .as_bytes()
        .to_vec();

        // header size is 2 bytes
        // the header contains the size of the whole msg
        let content_len = serialized.len();
        let mut msg: Vec<u8> = Vec::with_capacity(content_len + 2);
        msg.append(&mut u16::to_be_bytes(content_len as u16).to_vec());
        msg.append(&mut serialized);
        msg
    }

    /// expects to receive the msg without the header
    pub fn deserialize(s: &str) -> Result<Self, DeserializeError> {
        match s.split_once(":").ok_or(DeserializeError::InvalidFormat)? {
            ("ExecuteLua", rest) => match rest.splitn(2, ':').collect::<Vec<&str>>().as_slice() {
                &[print_type, code] => Ok(Message::ExecuteLua {
                    print_type: print_type
                        .parse::<bool>()
                        .map_err(|_| DeserializeError::InvalidFormat)?,
                    code: code.to_string(),
                }),
                _ => Err(DeserializeError::InvalidFormat),
            },
            ("GetBarContent", _) => Ok(Message::GetBarContent),
            ("GetState", _) => Ok(Message::GetState),
            _ => Err(DeserializeError::InvalidFormat),
        }
    }
}
