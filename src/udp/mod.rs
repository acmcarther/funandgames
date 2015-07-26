pub use self::udp::{
  start_network,
};

mod constants;
mod types;
mod errors;

mod udp {
  use std::net::{SocketAddr, UdpSocket};
  use std::sync::mpsc::{channel, Sender, Receiver};
  use std::thread;

  use udp::errors::{socket_bind_err, socket_recv_err, socket_send_err};
  use types::{
    SocketPayload,
    SequencedSocketPayload,
    SequencedAckedSocketPayload
  };
  use udp::types::{
    IOHandles,
    Network,
    RawSocketPayload
  };
  use udp::constants::{
    UDP_MARKER,
    PACKET_DROP_TIME
  };
  use byteorder::{ByteOrder, BigEndian};
  use std::collections::HashMap;
  use std::iter::{repeat};
  use helpers::{Tappable, TappableIter};
  use time::{Duration, PreciseTime};

  type OwnAcks = (SocketAddr, u16, u32);
  type DroppedPacket = (SocketAddr, u16);

  pub fn start_network(addr: SocketAddr) -> Network {

    let (send_tx, send_rx) = channel();
    let (recv_tx, recv_rx) = channel();

    let (send_attempted_tx, send_attempted_rx) = channel();
    let (dropped_packet_tx, dropped_packet_rx) = channel();
    //let (received_tx, received_rx) = channel();

    let send_socket =
      UdpSocket::bind(addr)
        .map_err(socket_bind_err)
        .unwrap();

    let recv_socket = send_socket.try_clone().unwrap();

    // Sending messages
    let send_handle = thread::spawn (move || {
      let mut seq_num_map = HashMap::new();
      loop {
        //let acks = try_recv_all(&ack_rx);
        try_recv_all(&dropped_packet_rx)
          .into_iter()
          .map(|final_payload: SequencedAckedSocketPayload| SocketPayload {addr: final_payload.addr, bytes: final_payload.bytes})
          .map(|raw_payload: SocketPayload| {
            let addr = raw_payload.addr.clone();
            (raw_payload, increment_seq_number(&mut seq_num_map, addr))
          })
          .map(|(raw_payload, seq_num)| add_sequence_number(raw_payload, seq_num))
          .map(|raw_payload: SequencedSocketPayload| {
            (raw_payload, 5, 5) // Fake ack_num for now
          })
          .map(|(payload, ack_num, ack_field)| add_acks(payload, ack_num, ack_field))
          .tap(|final_payload: &SequencedAckedSocketPayload| send_attempted_tx.send((final_payload.clone(), PreciseTime::now())))
          .map(serialize)
          .map(|raw_payload: RawSocketPayload| send_socket.send_to(raw_payload.bytes.as_slice(), raw_payload.addr))
          .map(|send_res| {send_res.map_err(socket_send_err);})
          .collect::<Vec<()>>();    // TODO: Remove collect

        let _ = send_rx.recv()
          .map(|raw_payload: SocketPayload| {
            let addr = raw_payload.addr.clone();
            (raw_payload, increment_seq_number(&mut seq_num_map, addr))
          })
          .map(|(raw_payload, seq_num)| add_sequence_number(raw_payload, seq_num))
          .map(|raw_payload: SequencedSocketPayload| {
            (raw_payload, 5, 5) // Fake ack_num for now
          })
          .map(|(payload, ack_num, ack_field)| add_acks(payload, ack_num, ack_field))
          .tap(|final_payload| send_attempted_tx.send((final_payload.clone(), PreciseTime::now())))
          .map(serialize)
          .map(|raw_payload: RawSocketPayload| send_socket.send_to(raw_payload.bytes.as_slice(), raw_payload.addr))
          .map(|send_res| send_res.map_err(socket_send_err));
      }
    });

    // Receiving messages
    let recv_handle = thread::spawn (move || {
      let mut packets_awaiting_ack = HashMap::new();
      loop {
        let mut buf = [0; 256];

        let now = PreciseTime::now();
        // Notify send thread of dropped packets
        //   Get keys first to sate the borrow checker
        let dropped_packet_keys: Vec<(SocketAddr, u16)> =
          packets_awaiting_ack.iter()
            .filter(|&(_, &(_, timestamp))| {
              let timestamp: PreciseTime = timestamp; // Compiler why?
              let time_elapsed: Duration = timestamp.to(now);
              time_elapsed.num_seconds() > PACKET_DROP_TIME
            })
            .map(|(key, &(_, _))| {
              let key: &(SocketAddr, u16) = key; // Compiler why?
              key.clone()
            }).collect();

        dropped_packet_keys.iter()
          .map(|key| packets_awaiting_ack.remove(&key))
          .filter(|result| result.is_some())
          .map(|result| result.unwrap())
          .map(|(packet, _)| {let _ = dropped_packet_tx.send(packet);}).collect::<Vec<()>>(); // TODO: Remove collect


        // Add all new sent packets packets_awaiting_ack
        try_recv_all(&send_attempted_rx)
          .into_iter()
          .map(|(sent_packet, timestamp)| {
            packets_awaiting_ack.insert(
              (sent_packet.addr.clone(), sent_packet.seq_num.clone()),
              (sent_packet, timestamp)
            );
          }).collect::<Vec<()>>();   // TODO: Get rid of this collect

        let payload_result = recv_socket.recv_from(&mut buf)
          .map_err(socket_recv_err)
          .map(|(_, socket_addr)| RawSocketPayload {addr: socket_addr, bytes: buf.to_vec()})
          .map(starts_with_marker);

        let _ = payload_result.map(|possible_payload| {
          possible_payload
            .map(strip_marker)
            .map(strip_sequence)
            .map(strip_acks)
            .tap(|packet| delete_acked_packets(&packet, &mut packets_awaiting_ack))
            .map(|payload| recv_tx.send(SocketPayload{addr: payload.addr, bytes: payload.bytes}))
        });
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

  fn add_acks(payload: SequencedSocketPayload, ack_num: u16, ack_field: u32) -> SequencedAckedSocketPayload {
    SequencedAckedSocketPayload {
      addr: payload.addr,
      seq_num: payload.seq_num,
      ack_num: ack_num,
      ack_field: ack_field,
      bytes: payload.bytes
    }
  }

  fn serialize(payload: SequencedAckedSocketPayload) -> RawSocketPayload {
    let mut sequence_num_bytes = [0; 2];
    let mut ack_num_bytes = [0; 2];
    let mut ack_field_bytes = [0; 4];
    BigEndian::write_u16(&mut sequence_num_bytes, payload.seq_num);
    BigEndian::write_u16(&mut ack_num_bytes, payload.ack_num);
    BigEndian::write_u32(&mut ack_field_bytes, payload.ack_field);
    let marked_and_seq_bytes: Vec<u8> =
      UDP_MARKER.into_iter().cloned()
        .chain(sequence_num_bytes.iter().cloned())
        .chain(ack_num_bytes.iter().cloned())
        .chain(ack_field_bytes.iter().cloned())
        .chain(payload.bytes.iter().cloned()).collect();
    RawSocketPayload {addr: payload.addr, bytes: marked_and_seq_bytes}
  }

  fn strip_marker(payload: RawSocketPayload) -> SocketPayload {
    SocketPayload { addr: payload.addr, bytes: payload.bytes[3..256].into_iter().cloned().collect() }
  }

  fn strip_sequence(payload: SocketPayload) -> SequencedSocketPayload {
    let seq_num = BigEndian::read_u16(&payload.bytes[0..2]);
    SequencedSocketPayload {
      addr: payload.addr,
      seq_num: seq_num,
      bytes: payload.bytes[2..253].into_iter().cloned().collect()
    }
  }

  fn strip_acks(payload: SequencedSocketPayload) -> SequencedAckedSocketPayload {
    let ack_num = BigEndian::read_u16(&payload.bytes[0..2]);
    let ack_field = BigEndian::read_u32(&payload.bytes[2..6]);
    SequencedAckedSocketPayload {
      addr: payload.addr,
      seq_num: payload.seq_num,
      ack_num: ack_num,
      ack_field: ack_field,
      bytes: payload.bytes[6..251].into_iter().cloned().collect()
    }
  }

  fn try_recv_all<T>(ack_rx: &Receiver<T>) -> Vec<T> {
    repeat(()).map(|_| ack_rx.try_recv().ok())
      .take_while(|x| x.is_some())
      .map(|x| x.unwrap())
      .collect()
  }

  fn delete_acked_packets(packet: &SequencedAckedSocketPayload, packets_awaiting_ack: &mut HashMap<(SocketAddr, u16), (SequencedAckedSocketPayload, PreciseTime)>) {
    let ack_num = packet.ack_num;
    let ack_field = packet.ack_field;
    let past_acks = (0..32).map(|bit_idx| {
      // Builds a bit mask, and checks if bit is present by comparing result to 0
      (bit_idx, 0 != ((1 << bit_idx) & ack_field))
    });

    // Remove initial ack
    packets_awaiting_ack.remove(&(packet.addr, ack_num));

    // Remove subsequent acks
    past_acks.map(|(idx, was_acked)| {
      if was_acked {
        packets_awaiting_ack.remove(&(packet.addr, ack_num - idx));
      }
    }).collect::<Vec<()>>(); // TODO: no collect
  }

}
