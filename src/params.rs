pub use self::params::{
  query_server_params,
  query_client_params,
  ClientParams,
  ServerParams,
};

mod params {
  use str_ops::{default_string, translate_localhost};
  use std::io::stdin;

  #[derive(Clone)]
  pub struct ClientParams {
    pub server_addr: String,
    pub server_port: i32,
    pub client_port: i32,
  }

  #[derive(Clone)]
  pub struct ServerParams {
    pub server_port: i32
  }

  pub fn query_server_params() -> ServerParams {
    let mut stdin = stdin();
    let mut port_str = String::new();
    println!("Server Port (5555): ");
    stdin.read_line(&mut port_str);
    port_str = default_string(port_str.trim(), "5555");

    ServerParams { server_port: i32::from_str_radix(&port_str.trim(), 10).unwrap() }
  }

  pub fn query_client_params() -> ClientParams {
    let mut stdin = stdin();
    let mut client_port_str = String::new();
    let mut server_port_str = String::new();
    let mut server_addr_str = String::new();
    println!("Client Port (4444): ");
    stdin.read_line(&mut client_port_str);
    println!("Server Port (5555): ");
    stdin.read_line(&mut server_port_str);
    println!("Server Addr (localhost): ");
    stdin.read_line(&mut server_addr_str);
    client_port_str = default_string(client_port_str.trim(), "4444");
    server_port_str = default_string(server_port_str.trim(), "5555");
    server_addr_str = translate_localhost(&default_string(server_addr_str.trim(), "localhost"));

    ClientParams {
      client_port: i32::from_str_radix(&client_port_str.trim(), 10).unwrap(),
      server_port: i32::from_str_radix(&server_port_str.trim(), 10).unwrap(),
      server_addr: server_addr_str.trim().to_string()
    }
  }
}
