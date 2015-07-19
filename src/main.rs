#![feature(convert)]
#![feature(lookup_host)]
#![feature(ip_addr)]
use std::env;
use std::io::stdin;
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};

mod app_net;
mod errors;
mod net_helpers;
mod params;
mod str_ops;
mod types;
mod udp;

use app_net::{add_user_and_broadcast, stringify_body};
use params::{
  query_server_params,
  query_client_params,
};
use types::{NetMode, ServerState, SocketPayload};
use str_ops::{net_mode_from_string};
use udp::start_network;

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
  let mut server_state = ServerState { users: HashMap::new() };
  println!("Server Started on {}", server_params.addr);

  let network = start_network(server_params.addr);

  let send_tx: Sender<SocketPayload> = network.send_channel;
  let recv_rx: Receiver<SocketPayload> = network.recv_channel;

  let chat_handle = thread::spawn (move || {
    loop {
      let socket_payload = recv_rx.recv().unwrap();
      add_user_and_broadcast(&mut server_state, socket_payload, &send_tx);
    }
  });

  let _ = vec![
    chat_handle.join(),
    network.thread_handles.send_handle.join(),
    network.thread_handles.recv_handle.join()
  ];
}

fn client() {
  let client_params = query_client_params();
  let server_addr = client_params.server_addr;
  println!("Client Started on {}", client_params.addr);
  println!("     to server on {}", client_params.server_addr);

  let network = start_network(client_params.addr);

  let send_tx: Sender<SocketPayload> = network.send_channel;
  let recv_rx: Receiver<SocketPayload> = network.recv_channel;

  let stdin_handle = thread::spawn (move || {
    let mut stdin = stdin();
    println!("type your messages");
    loop {
      let mut message = String::new();
      let _ = stdin.read_line(&mut message);
      let _ = send_tx.send((server_addr, message.as_bytes().into_iter().cloned().collect())).unwrap();
    }
  });

  let stdout_handle = thread::spawn (move || {
    loop {
      let socket_payload = recv_rx.recv().unwrap();
      let string_payload = stringify_body(socket_payload);
      let (_, string)  = string_payload;
      println!("{}", string.trim());
    }
  });

  let _ = vec![
    stdout_handle.join(),
    stdin_handle.join(),
    network.thread_handles.send_handle.join(),
    network.thread_handles.recv_handle.join(),
  ];
}
