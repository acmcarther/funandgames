pub use self::udp::constants::{
  UDP_MARKER,
  UDP_MARKER_LEN
};

mod constants {
  pub const UDP_MARKER: &'static [u8] = b"012";
  pub const UDP_MARKER_LEN: &'static u32 = 3;
}
