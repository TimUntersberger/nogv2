pub use nog_protocol::{json, BarContent, BarItem, BarItemAlignment, Message, State};
pub use std::net::ToSocketAddrs;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    time::Duration,
};

#[derive(Debug)]
pub enum ClientError {
    IoError(io::Error),
    LuaExecutionFailed(String),
    InvalidResponse(String),
}

pub type ClientResult<T = ()> = Result<T, ClientError>;

#[derive(Debug)]
pub struct Client {
    addr: String,
    stream: TcpStream,
}

impl Client {
    pub fn connect(addr: String, timeout: Option<Duration>) -> io::Result<Self> {
        let stream = match timeout {
            Some(timeout) => {
                TcpStream::connect_timeout(&addr.to_socket_addrs()?.next().unwrap(), timeout)?
            }
            None => TcpStream::connect(addr.clone())?,
        };
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        stream.set_write_timeout(Some(Duration::from_secs(1)))?;

        Ok(Self { stream, addr })
    }

    pub fn reconnect(&mut self) -> io::Result<()> {
        self.stream = TcpStream::connect(self.addr.clone())?;
        self.stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        self.stream
            .set_write_timeout(Some(Duration::from_secs(2)))?;

        Ok(())
    }

    pub fn send_message(&mut self, msg: &Message) -> io::Result<String> {
        self.stream.write_all(&msg.serialize())?;

        let mut response_header = [0u8; 2];
        self.stream.read_exact(&mut response_header)?;
        let response_len = u16::from_be_bytes(response_header);

        let mut response_body = vec![0u8; response_len as usize];
        self.stream.read_exact(&mut response_body)?;

        Ok(String::from_utf8(response_body).unwrap())
    }

    /// if `print_type` is set to `false` nog won't return the type of the statemnt
    pub fn execute_lua(&mut self, code: String, print_type: bool) -> ClientResult<String> {
        let response = self
            .send_message(&Message::ExecuteLua { code, print_type })
            .map_err(ClientError::IoError)?;

        match response
            .split_once(":")
            .ok_or_else(|| ClientError::InvalidResponse(response.clone()))?
        {
            ("Ok", output) => Ok(output.to_string()),
            ("Err", msg) => Err(ClientError::LuaExecutionFailed(msg.to_string())),
            _ => Err(ClientError::InvalidResponse(response)),
        }
    }

    pub fn get_bar_content(&mut self) -> ClientResult<BarContent> {
        let response = self
            .send_message(&Message::GetBarContent)
            .map_err(ClientError::IoError)?;

        json::from_str(&response).map_err(|_| ClientError::InvalidResponse(response))
    }

    pub fn get_state(&mut self) -> ClientResult<State> {
        let response = self
            .send_message(&Message::GetState)
            .map_err(ClientError::IoError)?;

        json::from_str(&response).map_err(|_| ClientError::InvalidResponse(response))
    }
}
