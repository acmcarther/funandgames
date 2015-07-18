pub use self::app_net::{
  add_user_and_broadcast,
  add_user,
  reply_all,
  stringify_body,
};

mod app_net {
  use std::sync::mpsc::Sender;
  use std::net::SocketAddr;

  use types::{
    ServerState,
    SocketPayload,
    StringPayload,
  };

  pub fn add_user_and_broadcast(server_state: &mut ServerState, payload: SocketPayload, send_tx: &Sender<SocketPayload>) {
    let (socket_addr, _) = payload;
    add_user(server_state, socket_addr);
    reply_all(server_state, payload, send_tx);
  }

  pub fn add_user(server_state: &mut ServerState, socket_addr: SocketAddr) {
    let key = socket_addr.to_string();
    if !server_state.users.contains_key(&key) {
      println!("saving user");
      server_state.users.insert(key, socket_addr.clone());
    }
  }

  pub fn reply_all(server_state: &ServerState, payload: SocketPayload, send_tx: &Sender<SocketPayload>) {
    let (from_socket_addr, payload_bytes) = payload;
    let from_socket_addr_str = from_socket_addr.to_string();
    let name_with_colon = from_socket_addr_str.clone() + ": ";
    let name_bytes = name_with_colon.as_bytes();
    let full_payload_bytes: Vec<u8> = name_bytes.into_iter().cloned().chain(payload_bytes.iter().cloned()).collect();
    server_state.users.values().map (|socket_addr| {
      if from_socket_addr_str != socket_addr.to_string() {
        send_tx.send((socket_addr.clone(), full_payload_bytes.clone()));
      }
    }).collect::<Vec<()>>();
  }

  pub fn stringify_body(payload: SocketPayload) -> StringPayload {
    let (socket_addr, bytes) = payload;
    let full_msg = String::from_utf8(bytes).unwrap();
    (socket_addr, full_msg.trim_matches('\0').to_string())
  }
}
