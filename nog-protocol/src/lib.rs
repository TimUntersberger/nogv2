use serde::{Deserialize, Serialize};

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
    pub bg: [f32; 3],
    pub items: Vec<BarItem>,
}

#[derive(Debug)]
pub enum Message {
    ExecuteLua { code: String },
    GetBarContent,
}

pub enum DeserializeError {
    InvalidFormat,
}

impl Message {
    pub fn serialize(&self) -> Vec<u8> {
        use Message::*;

        let mut serialized = match self {
            ExecuteLua { code } => format!("ExecuteLua:{}", code),
            GetBarContent => format!("GetBarContent:"),
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
            ("ExecuteLua", code) => Ok(Message::ExecuteLua {
                code: code.to_string(),
            }),
            ("GetBarContent", _) => Ok(Message::GetBarContent),
            _ => Err(DeserializeError::InvalidFormat),
        }
    }
}
