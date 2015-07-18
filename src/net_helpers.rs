pub use self::net_helpers::{
  get_own_ip
};

mod net_helpers {
  use std::net::{IpAddr, lookup_host, TcpStream, Shutdown};

  // TODO: Use a non-hacky solution
  pub fn get_own_ip() -> IpAddr {
    let external_ip =
      lookup_host("google.com")
        .unwrap().next()
        .unwrap().unwrap().ip();

    let stream = TcpStream::connect((external_ip, 80)).unwrap();
    let local_addr = stream.local_addr().unwrap();
    let _ = stream.shutdown(Shutdown::Both);
    local_addr.ip()
  }
}
