pub use self::app_net::{
  broadcast,
  stringify_body,
};

mod app_net {
  use std::sync::mpsc::Sender;
  use std::net::SocketAddr;

  use time::PreciseTime;

  use types::{
    ServerState,
    SocketPayload,
    StringPayload,
  };

  pub fn broadcast(server_state: &ServerState, payload: SocketPayload, send_tx: &Sender<SocketPayload>) {
    let (socket_addr, _) = payload;
    reply_all(server_state, payload, send_tx);
  }

  fn reply_all(server_state: &ServerState, payload: SocketPayload, send_tx: &Sender<SocketPayload>) {
    let (from_socket_addr, payload_bytes) = payload;
    let from_socket_addr_str = from_socket_addr.to_string();
    let name_with_colon = from_socket_addr_str.clone() + ": ";
    let name_bytes = name_with_colon.as_bytes();
    let full_payload_bytes: Vec<u8> = name_bytes.into_iter().cloned().chain(payload_bytes.iter().cloned()).collect();
    server_state.users.keys().map (|socket_addr| {
      if from_socket_addr_str != socket_addr.to_string() {
        let _ = send_tx.send((socket_addr.clone(), full_payload_bytes.clone()));
      }
    }).collect::<Vec<()>>();
  }

  pub fn stringify_body(payload: SocketPayload) -> StringPayload {
    let (socket_addr, bytes) = payload;
    let full_msg = String::from_utf8(bytes).unwrap();
    (socket_addr, full_msg.trim_matches('\0').to_string())
  }
}
