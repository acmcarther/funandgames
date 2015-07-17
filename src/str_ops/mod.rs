pub use self::str_ops::{default_string, translate_localhost};

mod str_ops {
  use constants::{LOCAL_IP, CODE_WORD};

  pub fn default_string(string: &str, default: &str) -> String {
    if string == "" {
      default.to_string()
    } else {
      string.to_string()
    }
  }

  pub fn translate_localhost(string: &str) -> String {
    if string == "localhost" {
      LOCAL_IP.to_string()
    } else {
      string.to_string()
    }
  }
}
