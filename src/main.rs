#![feature(lookup_host)]
use std::env;
use std::ascii::AsciiExt;
use std::io::stdin;
use std::net::{SocketAddr, UdpSocket};
use std::io::Error;
use std::sync::mpsc::channel;
use std::thread;
use std::collections::HashMap;

mod str_ops;
mod params;
mod constants;
use params::{
  query_server_params,
  query_client_params,
  ClientParams,
  ServerParams,
};
use str_ops::{default_string, translate_localhost};
use constants::{LOCAL_IP, CODE_WORD};

#[derive(Debug)]
enum NetMode {
  Client,
  Server
}

struct ServerState {
  users: HashMap<String, SocketAddr>
}

fn main() {
  let host:Vec<SocketAddr> = std::net::lookup_host("rust-lang.org").unwrap().map(|x| x.unwrap()).collect();
  println!("host: {:?}", host);
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
  let full_addr_string: String = LOCAL_IP.to_string() + ":" + &port;
  let full_addr: &str = &full_addr_string;

  UdpSocket::bind(full_addr)
    .map_err(log_error)
    .map(be_server);
}

fn client() {
  let client_params = query_client_params();
  let client_port = client_params.client_port.to_string();
  let full_client_addr_string: String = LOCAL_IP.to_string() + ":" + &client_port;
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
  let mut server_state = ServerState { users: HashMap::new() };
  loop {
    let mut buf = [0; 256];
    socket.recv_from(&mut buf)
      .map_err(log_error)
      .map(|(_, socket_addr)| (socket_addr, stringify_bytes(&buf)))
      .map(|(socket_addr, msg)| filter_msg_for_relevance(socket_addr, msg))
      .map(|possible_payload| add_user(&mut server_state, possible_payload))
      .map(|possible_payload| reply_all(&server_state, &socket, possible_payload));
  }
}

fn be_client(socket: UdpSocket, client_params: ClientParams) {
  let (stdin_tx, stdin_rx) = channel();
  let (send_tx, send_rx) = channel();
  let recv_socket = socket.try_clone().unwrap();

  // Console IO
  thread::spawn (move || {
    let mut stdin = stdin();
    loop {
      let mut message = String::new();

      println!("type your message!");
      stdin.read_line(&mut message);
      let coded_trimmed_message = CODE_WORD.to_string() + message.trim();
      stdin_tx.send(coded_trimmed_message).unwrap();
    }
  });

  // Sending messages
  thread::spawn (move || {
    let server_addr = client_params.server_addr.to_string();
    let server_port = client_params.server_port.to_string();
    let full_server_addr_string: String = server_addr + ":" + &server_port;
    let full_server_addr: &str = full_server_addr_string.as_ref();

    loop {
      let message_to_deliver: String = send_rx.recv().unwrap();
      socket.send_to(message_to_deliver.as_bytes(), full_server_addr)
        .map_err(log_error);
    }
  });

  // Receiving messages
  thread::spawn (move || {
    loop {
      let mut buf = [0; 256];
      recv_socket.recv_from(&mut buf)
        .map(|(_, socket_addr)| (socket_addr, stringify_bytes(&buf)))
        .map(|(socket_addr, msg)| filter_msg_for_relevance(socket_addr, msg))
        .map(log_response);
    }
  });

  // The bucket brigade
  loop {
    let res = stdin_rx.recv().unwrap();
    send_tx.send(res).unwrap();
  }
}

fn add_user(server_state: &mut ServerState, possible_payload: Option<(SocketAddr, String)>) -> Option<String> {
  println!("add user");
  possible_payload.map(|(socket_addr, msg)| {
    let key = socket_addr.to_string();
    if !server_state.users.contains_key(&key) {
      println!("saving user");
      server_state.users.insert(key, socket_addr.clone());
    }
    socket_addr.to_string() + ": " + &msg
  })
}

fn reply_all(server_state: &ServerState, socket: &UdpSocket, possible_payload: Option<String>) {
  possible_payload.map(|msg| {
    println!("{:?}", server_state.users);
    let lotsOfNothing: Vec<()> = server_state.users.values().map (|socketAddr| {
      println!("replying");
      reply(socket, socketAddr, &msg)
    }).collect();
  });
}

fn reply(socket: &UdpSocket, socket_addr: &SocketAddr, msg: &String, ) {
  println!("received message [{}] from {:?}", msg, socket_addr);
  let return_msg = deduce_return_msg(&msg);
  println!("now sending [{}] to {}", return_msg, socket_addr);
  socket.send_to(return_msg.as_bytes(), socket_addr);
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

fn deduce_return_msg(msg: &String) -> String {
  let code_word_string = CODE_WORD.to_string();
  code_word_string + msg
}



fn log_response(possible_payload: Option<(SocketAddr, String)>) {
  possible_payload.map(|(socket_addr, msg)| {
    println!("received message [{}] from {:?}", msg, socket_addr);
  });
}

fn log_error(err: Error) {
  println!("Connection Error: {}", err)
}

