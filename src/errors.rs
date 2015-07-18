pub use self::errors::{
  socket_bind_err,
};

mod errors {
  use std::io::Error;

  pub fn socket_bind_err(err: Error) {
    println!("UDP: Error binding socket: {}", err)
  }

}
