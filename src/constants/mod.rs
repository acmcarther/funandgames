pub use self::constants::{
  UDP_MARKER,
  LOCAL_IP
};

mod constants {
  pub const UDP_MARKER: &'static [u8] = b"012";
  pub const LOCAL_IP: &'static str = "192.168.129.84";
}
