pub use self::types::{
  SocketPayload,
  SequencedSocketPayload,
  SequencedAckedSocketPayload,
  IdentifiedPayload,
  StringPayload,
  NetMode,
  ServerState,
  MessageType,
  message_type_to_byte,
  byte_to_message_type
};

mod types {
  use std::net::SocketAddr;
  use connected_udp::ConnectionTable;

  #[derive(Clone)]
  pub struct SocketPayload {
    pub addr: SocketAddr,
    pub bytes: Vec<u8>
  }

  #[derive(Clone)]
  pub struct SequencedSocketPayload {
    pub addr: SocketAddr,
    pub seq_num: u16,
    pub bytes: Vec<u8>
  }

  #[derive(Clone)]
  pub struct SequencedAckedSocketPayload {
    pub addr: SocketAddr,
    pub seq_num: u16,
    pub ack_num: u16,
    pub ack_field: u32,
    pub bytes: Vec<u8>
  }

  #[derive(Clone)]
  pub struct StringPayload {
    pub addr: SocketAddr,
    pub msg: String
  }

  #[derive(Clone)]
  pub struct IdentifiedPayload {
    pub addr: SocketAddr,
    pub msg_type: MessageType,
    pub bytes: Vec<u8>
  }

  #[derive(Debug, Clone)]
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
