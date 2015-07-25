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
  use types::{
    SocketPayload,
    SequencedSocketPayload
  };
  use udp::types::{
    IOHandles,
    Network,
    RawSocketPayload
  };
  use udp::constants::UDP_MARKER;
  use byteorder::{ByteOrder, BigEndian};
  use std::collections::HashMap;

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
      let mut seq_num_map = HashMap::new();
      loop {
        let _ = send_rx.recv()
          .map(|raw_payload: SocketPayload| {
            let addr = raw_payload.addr.clone();
            (raw_payload, increment_seq_number(&mut seq_num_map, addr))
          })
          .map(|(raw_payload, seq_num)| add_sequence_number(raw_payload, seq_num))
          .map(serialize)
          .map(|raw_payload: RawSocketPayload| send_socket.send_to(raw_payload.bytes.as_slice(), raw_payload.addr))
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
          .map(|payload| payload.map(strip_sequence))
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

  fn increment_seq_number(seq_num_map: &mut HashMap<SocketAddr, u16>, addr: SocketAddr) -> u16 {
    let count = seq_num_map.entry(addr).or_insert(0);
    *count += 1;
    count.clone()
  }

  fn add_sequence_number(payload: SocketPayload, sequence_num: u16) -> SequencedSocketPayload {
    SequencedSocketPayload { addr: payload.addr, seq_num: sequence_num, bytes: payload.bytes }
  }

  fn serialize(payload: SequencedSocketPayload) -> RawSocketPayload {
    let mut sequence_num_bytes = [0; 2];
    BigEndian::write_u16(&mut sequence_num_bytes, payload.seq_num);
    let marked_and_seq_bytes: Vec<u8> =
      UDP_MARKER.into_iter().cloned()
        .chain(sequence_num_bytes.iter().cloned())
        .chain(payload.bytes.iter().cloned()).collect();
    RawSocketPayload {addr: payload.addr, bytes: marked_and_seq_bytes}
  }

  fn strip_marker(payload: RawSocketPayload) -> SocketPayload {
    SocketPayload { addr: payload.addr, bytes: payload.bytes[3..256].into_iter().cloned().collect() }
  }

  fn strip_sequence(payload: SocketPayload) -> SocketPayload {
    // TODO: Use this value
    let seq_num = BigEndian::read_u16(&payload.bytes[0..2]);
    println!("seq_num: {}", seq_num);
    SocketPayload { addr: payload.addr, bytes: payload.bytes[2..253].into_iter().cloned().collect() }
  }
}
