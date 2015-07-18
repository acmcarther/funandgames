pub use self::str_ops::{
  default_string,
  translate_localhost,
  net_mode_from_string,
};

mod str_ops {
  use std::ascii::AsciiExt;
  use types::NetMode;
  use net_helpers::get_own_ip;

  pub fn default_string(string: &str, default: &str) -> String {
    if string == "" {
      default.to_string()
    } else {
      string.to_string()
    }
  }

  pub fn translate_localhost(string: &str) -> String {
    if string == "localhost" {
      let local_ip = get_own_ip();
      local_ip.to_string()
    } else {
      string.to_string()
    }
  }

  pub fn net_mode_from_string(mode_str: &str) -> NetMode {
    match mode_str.to_ascii_lowercase().as_ref() {
      "server" => NetMode::Server,
      _ => NetMode::Client
    }
  }
}
