pub use self::udp::payloads::{
  SocketPayload,
  SequencesPayload,
  AckedPayload,
  MarkedAckedPayload,
  OutboundPayload,
};

mod payloads {
  pub struct RawSocketPayload {
    pub addr: SocketAddr,
    pub payload: Vec<u8>
  }

  pub struct OutboundPayload {
    pub addr: SocketAddr,
    pub payload: Vec<u8>
  }

  pub struct SocketPayload {
    pub addr: SocketAddr,
    pub payload: Vec<u8>,
  }

  pub struct SequencedPayload {
    pub addr: SocketAddr,
    pub sequence_num: u16,
    pub payload: Vec<u8>
  }

  pub struct AckedPayload {
    pub addr: SocketAddr,
    pub sequence_num: u16,
    pub ack_num: u16,
    pub ack_field: BitVec, // bitfield identifying 32 acked packets
    pub payload: Vec<u8>
  }

  pub struct MarkedAckedPayload {
    pub addr: SocketAddr,
    pub marker: Vec<u8>,
    pub sequence_num: u16,
    pub ack_num: u16,
    pub ack_field: BitVec, // bitfield identifying 32 acked packets
    pub payload: Vec<u8>
  }

  pub impl MarkedAckedPayload {
    pub fn new(acked: AckedPayload, marker: Vec<u8>) -> MarkedAckedPayload {
      MarkedAckedPayload {
        addr: acked.addr,
        marker: marker,
        sequence_num: acked.sequence_num,
        ack_num: acked.ack_num,
        ack_field: acked.ack_field
        payload: acked.payload
      }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
      self.marker.clone().iter().chain(
        sequence_num.
        ack_num.
        ack_field.as_bytes()
    }
  }

}
