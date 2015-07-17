use std::env;
use std::ascii::AsciiExt;
use std::io::stdin;
use std::net::{SocketAddr, UdpSocket};
use std::io::Error;

#[derive(Debug)]
enum NetMode {
  Client,
  Server
}

#[derive(Clone)]
struct ClientParams {
  server_addr: String,
  server_port: i32,
  client_port: i32,
}

#[derive(Clone)]
struct ServerParams {
  server_port: i32
}

struct ServerState {
  users: Vec<User>
}

struct User {
  connection: SocketAddr
}

const CODE_WORD: &'static str = "funandgames";

fn main() {
  let second_arg = env::args().nth(1);
  let net_mode = net_mode_from_string(&second_arg.unwrap_or("client".to_string()));
  println!("You are {:?}", net_mode);
  match net_mode {
    NetMode::Server => server(),
    NetMode::Client => client()
  }
}

fn server() {
  let server_params = query_server_params();
  let port = server_params.server_port.to_string();
  let full_addr_string: String = "127.0.0.1:".to_string() + &port;
  let full_addr: &str = &full_addr_string;

  UdpSocket::bind(full_addr)
    .map_err(log_error)
    .map(be_server);
}

fn client() {
  let client_params = query_client_params();
  let client_port = client_params.client_port.to_string();
  let full_client_addr_string: String = "127.0.0.1:".to_string() + &client_port;
  let full_client_addr: &str = &full_client_addr_string;

  UdpSocket::bind(full_client_addr)
    .map_err(log_error)
    .map(|socket| be_client(socket, client_params));
}

fn net_mode_from_string(mode_str: &str) -> NetMode {
  match mode_str.to_ascii_lowercase().as_ref() {
    "server" => NetMode::Server,
    _ => NetMode::Client
  }
}

fn be_server(socket: UdpSocket) {
  println!("now hosting!");
  let mut server_state = ServerState { users: Vec::new() };
  loop {
    let mut buf = [0; 256];
    socket.recv_from(&mut buf)
      .map_err(log_error)
      .map(|(_, socket_addr)| (socket_addr, stringify_bytes(&buf)))
      .map(|(socket_addr, msg)| filter_msg_for_relevance(socket_addr, msg))
      .map(|possible_payload| reply(&socket, possible_payload));
  }
}

fn be_client(socket: UdpSocket, client_params: ClientParams) {
  loop {
    let server_addr = client_params.server_addr.to_string();
    let server_port = client_params.server_port.to_string();
    let full_server_addr_string: String = server_addr + ":" + &server_port;
    let full_server_addr: &str = full_server_addr_string.as_ref();
    let mut stdin = stdin();
    let mut message = String::new();

    println!("type your message!");
    stdin.read_line(&mut message);
    let coded_trimmed_message = CODE_WORD.to_string() + message.trim();

    println!("now sending [{}] to {}", coded_trimmed_message, full_server_addr);
    socket.send_to(coded_trimmed_message.as_bytes(), full_server_addr)
      .map_err(log_error)
      .map( |_| {
        let mut buf = [0; 256];
        println!("now waiting for response");
        socket.recv_from(&mut buf)
          .map(|(_, socket_addr)| (socket_addr, stringify_bytes(&buf)))
          .map(|(socket_addr, msg)| filter_msg_for_relevance(socket_addr, msg))
          .map(log_response);
      });
  }
}

fn reply(socket: &UdpSocket, possible_payload: Option<(SocketAddr, String)>) {
  possible_payload.map(|(socket_addr, msg)| {
    println!("received message [{}] from {:?}", msg, socket_addr);
    let return_msg = deduce_return_msg(msg);
    println!("now sending [{}] to {}", return_msg, socket_addr);
    socket.send_to(return_msg.as_bytes(), socket_addr)
  });
}

fn stringify_bytes(bytes: &[u8]) -> String {
  let full_msg = String::from_utf8(bytes.to_vec()).unwrap();
  full_msg.trim_matches('\0').to_string()
}

fn filter_msg_for_relevance(socket_addr: SocketAddr, msg: String) -> Option<(SocketAddr, String)> {
  if msg.starts_with(CODE_WORD) {
    let msg_body: String = msg.chars().skip(CODE_WORD.len()).collect();
    Some((socket_addr, msg_body))
  } else {
    None
  }
}

fn deduce_return_msg(msg: String) -> String {
  let code_word_string = CODE_WORD.to_string();
  if (&msg == "hello") {
    code_word_string + "goodbye"
  } else {
    code_word_string + "what u say?"
  }
}



fn log_response(possible_payload: Option<(SocketAddr, String)>) {
  possible_payload.map(|(socket_addr, msg)| {
    println!("received message [{}] from {:?}", msg, socket_addr);
  });
}

fn log_error(err: Error) {
  println!("Connection Error: {}", err)
}

fn query_server_params() -> ServerParams {
  let mut stdin = stdin();
  let mut port_str = String::new();
  println!("Server Port (5555): ");
  stdin.read_line(&mut port_str);
  port_str = default_string(port_str.trim(), "5555");

  ServerParams { server_port: i32::from_str_radix(&port_str.trim(), 10).unwrap() }
}

fn query_client_params() -> ClientParams {
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

fn default_string(string: &str, default: &str) -> String {
  if string == "" {
    default.to_string()
  } else {
    string.to_string()
  }
}

fn translate_localhost(string: &str) -> String {
  if string == "localhost" {
    "127.0.0.1".to_string()
  } else {
    string.to_string()
  }
}
