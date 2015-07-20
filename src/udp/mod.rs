pub use self::udp::{
  start_network,
};

mod constants;
mod types;
mod errors;

mod udp {
  use std::net::{SocketAddr, UdpSocket};
  use std::sync::mpsc::channel;
  use std::thread;

  use udp::errors::{socket_bind_err, socket_recv_err, socket_send_err};
  use types::SocketPayload;
  use udp::types::{
    IOHandles,
    Network,
    RawSocketPayload
  };
  use udp::constants::UDP_MARKER;


  pub fn start_network(addr: SocketAddr) -> Network {

    let (send_tx, send_rx) = channel();
    let (recv_tx, recv_rx) = channel();

    let send_socket =
      UdpSocket::bind(addr)
        .map_err(socket_bind_err)
        .unwrap();

    let recv_socket = send_socket.try_clone().unwrap();

    // Sending messages
    let send_handle = thread::spawn (move || {
      loop {
        let _ = send_rx.recv()
          .map(add_payload_marker)
          .map(|raw_payload| send_socket.send_to(raw_payload.bytes.as_slice(), raw_payload.addr))
          .map(|send_res| send_res.map_err(socket_send_err));
      }
    });

    // Receiving messages
    let recv_handle = thread::spawn (move || {
      loop {
        let mut buf = [0; 256];
        let _ = recv_socket.recv_from(&mut buf)
          .map_err(socket_recv_err)
          .map(|(_, socket_addr)| RawSocketPayload {addr: socket_addr, bytes: buf.to_vec()})
          .map(starts_with_marker)
          .map(|payload| payload.map(strip_marker))
          .map(|payload| payload.map(|val| recv_tx.send(val)));
      }
    });
    let io_handles = IOHandles { send_handle: send_handle, recv_handle: recv_handle };
    Network { send_channel: send_tx, recv_channel: recv_rx, thread_handles: io_handles }
  }

  fn starts_with_marker(payload: RawSocketPayload) -> Option<RawSocketPayload> {
    if &payload.bytes[0..3] == UDP_MARKER {
      Some(payload)
    } else {
      None
    }
  }

  fn add_payload_marker(payload: SocketPayload) -> RawSocketPayload {
    let marked_bytes: Vec<u8> = UDP_MARKER.into_iter().cloned().chain(payload.bytes.iter().cloned()).collect();
    RawSocketPayload {addr: payload.addr, bytes: marked_bytes}
  }

  fn strip_marker(payload: RawSocketPayload) -> SocketPayload {
    SocketPayload { addr: payload.addr, bytes: payload.bytes[3..256].into_iter().cloned().collect() }
  }
}
