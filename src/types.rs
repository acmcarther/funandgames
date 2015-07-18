pub use self::types::{
  SocketPayload,
  StringPayload,
  NetMode,
  ServerState,
};

mod types {
  use std::collections::HashMap;
  use std::net::SocketAddr;

  pub type SocketPayload = (SocketAddr, Vec<u8>);
  pub type StringPayload = (SocketAddr, String);

  #[derive(Debug)]
  pub enum NetMode {
    Client,
    Server
  }

  pub struct ServerState {
    pub users: HashMap<String, SocketAddr>
  }
}
