#[derive(Debug)]
pub enum Message {
    ExecuteLua {
        code: String
    }
}

impl Message {
    pub fn serialize(&self) -> Vec<u8> {
        use Message::*;

        let mut serialized = match self {
            ExecuteLua { code } => {
                format!("ExecuteLua:{}", code).as_bytes().to_vec()
            }
        };

        // header size is 2 bytes
        // the header contains the size of the whole msg
        let content_len = serialized.len();
        let mut msg: Vec<u8> = Vec::with_capacity(content_len + 2);
        msg.append(&mut u16::to_be_bytes(content_len as u16).to_vec());
        msg.append(&mut serialized);
        msg
    }

    /// expects to receive the msg without the header
    pub fn deserialize(s: &str) -> Result<Self, ()> {
        if let Some(code) = s.strip_prefix("ExecuteLua:") {
            return Ok(Message::ExecuteLua {
                code: code.to_string()
            })
        }

        Err(())
    }
}
