pub use self::udp::{
  start_network,
};

mod udp {
  use std::net::{SocketAddr, UdpSocket};
  use std::sync::mpsc::{Sender, Receiver};
  use std::io::Error;
  use std::thread::JoinHandle;
  use std::thread;

  use constants::{UDP_MARKER};
  use types::SocketPayload;

  type RawSocketPayload = (SocketAddr, Vec<u8>);

  pub fn start_network(
    socket: UdpSocket,
    send_rx: Receiver<SocketPayload>,
    recv_tx: Sender<Result<SocketPayload, Error>>,
    ) -> (JoinHandle< ()>, JoinHandle<()>) {

    let recv_socket = socket.try_clone().unwrap();

    // Sending messages
    let send_handle = thread::spawn (move || {
      loop {
        send_rx.recv()
          .map(add_payload_marker)
          .map(|(socket_addr, payload)| socket.send_to(payload.as_slice(), socket_addr));
      }
    });

    // Receiving messages
    let recv_handle = thread::spawn (move || {
      loop {
        let mut buf = [0; 256];
        let result = recv_socket.recv_from(&mut buf)
          .map(|(_, socket_addr)| (socket_addr, buf.to_vec()))
          .map(starts_with_marker)
          .map(|payload| payload.map(strip_marker));

        // Invert the monads, send if Some(res) or Err
        match result {
          Ok(opt) => opt.map (|val| Ok(val)),
          Err(err) => Some(Err(err))
        }.map(|res| recv_tx.send(res));
      }
    });

    (send_handle, recv_handle)
  }

  fn starts_with_marker(payload: RawSocketPayload) -> Option<RawSocketPayload> {
    let (socket_addr, payload) = payload;
    if &payload[0..3] == UDP_MARKER {
      Some((socket_addr, payload))
    } else {
      None
    }
  }

  fn add_payload_marker(payload: SocketPayload) -> RawSocketPayload {
    let (socket_addr, payload) = payload;
    let marked_bytes: Vec<u8> = UDP_MARKER.into_iter().cloned().chain(payload.iter().cloned()).collect();
    (socket_addr, marked_bytes)
  }

  fn strip_marker(payload: RawSocketPayload) -> SocketPayload {
    let (socket_addr, payload) = payload;
    (socket_addr, payload[3..256].into_iter().cloned().collect())
  }
}
