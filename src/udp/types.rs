pub use self::types::{
  IOHandles,
  Network,
  RawSocketPayload
};

mod types {
  use std::thread::JoinHandle;
  use types::SocketPayload;
  use std::sync::mpsc::{channel, Sender, Receiver};
  use std::net::{SocketAddr, UdpSocket};

  pub struct IOHandles {
    pub send_handle: JoinHandle<()>,
    pub recv_handle: JoinHandle<()>
  }

  pub struct Network {
    pub send_channel: Sender<SocketPayload>,
    pub recv_channel: Receiver<SocketPayload>,
    pub thread_handles: IOHandles
  }

  pub type RawSocketPayload = (SocketAddr, Vec<u8>);
}
