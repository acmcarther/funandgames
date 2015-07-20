pub use self::udp::types::{
  IOHandles,
  Network,
  PacketRecord,
  PacketRecords,
  PacketRecordMap,
  AckRecord,
  AckRecordMap,
  SequenceNumMap
};

mod types {
  use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
  use std::collections::HashMap;

  pub struct IOHandles {
    pub send_handle: JoinHandle<()>,
    pub recv_handle: JoinHandle<()>
  }

  pub struct Network {
    pub send_channel: Sender<OutboundPayload>,
    pub recv_channel: Receiver<AckedPayload>,
    pub thread_handles: IOHandles
  }

  pub type PacketRecord = (SocketAddr, u16);
  pub type PacketRecords = VecDeque<u16>;
  pub type PacketRecordMap = HashMap<SocketAddr, PacketRecords>;

  pub type AckRecord = (u16, BitVec);
  pub type AckRecordMap = HashMap<SocketAddr, AckRecord>;

  pub type SequenceNumMap = HashMap<SocketAddr, u16>;
}
