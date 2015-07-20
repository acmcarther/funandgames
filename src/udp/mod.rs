pub use self::udp::{
  start_network,
  AckedPayload,
  OutboundPayload
};

mod udp {
  use std::net::{SocketAddr, UdpSocket};
  use std::sync::mpsc::{channel, Sender, Receiver};
  use std::thread::JoinHandle;
  use std::thread;

  use errors::{socket_bind_err, socket_recv_err, socket_send_err};
  use udp::constants::{UDP_MARKER, UDP_MARKER_LEN};
  use udp::types::{
    Network,
    RawSocketPayload,
    SocketPayload,
    SequencedPayload,
    AckedPayload,
    OutboundPayload
  };

  use std::collections::hash_map::Entry::{OccupiedEntry, VacantEntry};

  pub fn start_network(addr: SocketAddr) -> Network {

    let (send_msg_tx, send_msg_rx) = channel();
    let (recv_msg_tx, recv_msg_rx) = channel();
    let (send_ack_tx, send_ack_rs) = channel();

    let send_socket =
      UdpSocket::bind(addr)
        .map_err(socket_bind_err)
        .unwrap();

    let recv_socket = send_socket.try_clone().unwrap();

    // Sending messages
    let send_handle = thread::spawn (move || {
      let mut ack_record_map: AckRecordMap   = HashMap::new();
      let mut seq_number_map: SequenceNumMap = HashMap::new();

      loop {
        // TODO: Implement
        ack_record_map = update_ack_records(ack_record_map, &send_acks_rx);

        // TODO: Resend unacked messages
        let _ = send_rx.recv()
          .map(|outbound_payload| {
            update_seq_numbers(&outbound_payload.socket_addr, &mut seq_number_map)
            outbound_payload
          })
          .map(|outbound_payload| add_ack_records(outbound_payload, &ack_record_map))
          .map(|outbound_payload| add_sequence_number(outbound_payload, &seq_number_map))
          .map(|outbound_payload| add_payload_marker(outbound_payload))
          .map(|outbound_payload| {
            send_socket.send_to(payload.into_bytes(), outbound_payload.addr)
          })
          .map(|send_res| send_res.map_err(socket_send_err));
      }
    });

    // Receiving messages
    // TODO: Define `tap` for `Option`
    let recv_handle = thread::spawn (move || {
      let mut packet_record_map: PacketRecordMap = HashMap::new();
      let mut own_ack_record_map: AckRecordMap = HashMap::new();

      loop {
        let mut buf = [0; 256];
        let _ = recv_socket.recv_from(&mut buf)
          .map_err(socket_recv_err)
          .map(|(_, socket_addr)| (socket_addr, buf.to_vec()))
          .map(starts_with_marker)
          .map(|payload| {
            payload
              .map(strip_marker)
              .map(sequence_payload)
              .map(ack_payload)
              .map(|payload| {
                update_packet_record_map(&payload.addr, &payload.sequence_num, &mut packet_record_map)
                update_own_acks(&payload.addr, &payload.sequence_num, &mut own_ack_record_map)
                payload
              })
              .map(|msg| recv_tx.send(msg))
          });
      }
    });


    let io_handles = IOHandles { send_handle: send_handle, recv_handle: recv_handle };
    Network { send_channel: send_tx, recv_channel: recv_rx, thread_handles: io_handles }
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

  fn sequence_payload(payload: SocketPayload) -> SequencedPayload {
  }

  fn ack_payload(payload: SequencedPayload) -> AckedPayload {
  }

  fn update_packet_record_map(addr: &SocketAddr, seq_num: &u16, packet_record_map: &mut PacketRecordMap) {
    let mut current_record_entry = packet_record_map.entry(socket_addr);
    match currrent_record_entry {
      OccupiedEntry(entry) => {
        let mut sequence_deq = entry.get_mut()
        sequence_deque.push_front(sequence_num);
        entry.insert(new_deque);
      },
      VacantEntry(entry) => {
        let new_deque = VecDeque::new();
        new_deque.push_front(sequence_num);
        entry.insert(new_deque);
      }
    }
  }

  fn update_own_acks(addr: &SocketAddr, seq_num: &u16, ack_record_map: &mut AckRecordMap) {
    let mut current_record_entry = packet_record_map.entry(socket_addr);
    match currrent_record_entry {
      OccupiedEntry(entry) => {
        let mut sequence_deq = entry.get_mut()
        sequence_deque.push_front(sequence_num);
        entry.insert(new_deque);
      },
      VacantEntry(entry) => {
        let new_deque = VecDeque::new();
        new_deque.push_front(sequence_num);
        entry.insert(new_deque);
      }
    }
  }

  fn update_seq_numbers(

}
