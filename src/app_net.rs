pub use self::app_net::{
  handle_payload,
  identify_payload,
  stringify_body,
};

mod app_net {
  use std::sync::mpsc::Sender;
  use std::net::SocketAddr;

  use types::{
    ServerState,
    IdentifiedPayload,
    StringPayload,
    MessageType,
    byte_to_message_type,
    message_type_to_byte
  };

  use game_udp::packet_types::Packet;

  pub fn identify_payload(payload: Packet) -> Option<IdentifiedPayload> {
    let mut bytes = payload.bytes;
    let addr = payload.addr;
    byte_to_message_type(bytes[0])
      .map(|message_type| {
        bytes.remove(0);
        IdentifiedPayload {addr: addr, msg_type: message_type, bytes: bytes }
       })
  }

  pub fn handle_payload(server_state: &ServerState, payload: IdentifiedPayload, send_tx: &Sender<Packet>) {
    match payload.msg_type {
      MessageType::Message => handle_message(server_state, &payload.addr, &payload.bytes, send_tx),
      _ => ()
    }
  }

  fn handle_message(server_state: &ServerState, origin: &SocketAddr, payload_bytes: &Vec<u8>, send_tx: &Sender<Packet>) {
    reply_all(server_state, origin, payload_bytes, send_tx);
  }


  fn reply_all(server_state: &ServerState, origin: &SocketAddr, payload_bytes: &Vec<u8>, send_tx: &Sender<Packet>) {
    let from_socket_addr_str = origin.to_string();
    let name_with_colon = from_socket_addr_str.clone() + ": ";
    let name_bytes = name_with_colon.as_bytes();
    let full_payload_bytes: Vec<u8> =
      [message_type_to_byte(MessageType::Message)].into_iter().cloned()
      .chain(name_bytes.into_iter().cloned())
      .chain(payload_bytes.iter().cloned()).collect();
    server_state.users.keys().map (|socket_addr| {
      if from_socket_addr_str != socket_addr.to_string() {
        let _ = send_tx.send(Packet {addr: socket_addr.clone(), bytes: full_payload_bytes.clone()});
      } else {
        let _ = send_tx.send(Packet{addr: socket_addr.clone(), bytes: vec![message_type_to_byte(MessageType::KeepAlive)]});
      }
    }).collect::<Vec<()>>();
  }

  pub fn stringify_body(bytes: Vec<u8>) -> String {
    let full_msg = String::from_utf8(bytes).unwrap();
    full_msg.trim_matches('\0').to_string()
  }
}
