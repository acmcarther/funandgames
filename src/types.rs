pub use self::types::{
  SocketPayload,
  IdentifiedPayload,
  StringPayload,
  NetMode,
  ServerState,
  MessageType,
  message_type_to_byte,
  byte_to_message_type
};

mod types {
  use std::collections::HashMap;
  use std::net::SocketAddr;

  use connected_udp::ConnectionTable;

  pub type SocketPayload = (SocketAddr, Vec<u8>);
  pub type StringPayload = (SocketAddr, String);
  pub type IdentifiedPayload = (SocketAddr, MessageType, Vec<u8>);

  #[derive(Debug)]
  pub enum MessageType {
    KeepAlive,
    Message,
  }

  #[derive(Debug)]
  pub enum NetMode {
    Client,
    Server
  }

  pub struct ServerState {
    pub users: ConnectionTable
  }

  pub fn message_type_to_byte(msg_type: MessageType) -> u8 {
    match msg_type {
      MessageType::KeepAlive => 1,
      MessageType::Message => 2,
    }
  }

  pub fn byte_to_message_type(byte: u8) -> Option<MessageType> {
    match byte {
      1 => Some(MessageType::KeepAlive),
      2 => Some(MessageType::Message),
      _ => None
    }
  }
}
