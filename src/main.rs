#![feature(convert)]
#![feature(lookup_host)]
#![feature(ip_addr)]
use std::env;
use std::io::stdin;
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};

use time::PreciseTime;

extern crate time;
extern crate byteorder;

mod app_net;
mod connected_udp;
mod errors;
mod net_helpers;
mod params;
mod str_ops;
mod types;
mod udp;

use app_net::{
  identify_payload,
  handle_payload,
  stringify_body
};
use params::{
  query_server_params,
  query_client_params,
};
use types::{message_type_to_byte, MessageType, NetMode, ServerState, SocketPayload, IdentifiedPayload};
use str_ops::{net_mode_from_string};
use udp::start_network;
use connected_udp::{handle_connections, cull_connections};

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
    let mut last_call = PreciseTime::now();
    loop {
      let _ = recv_rx.recv()
        .map(|payload| handle_connections(payload, &mut server_state.users))

        .map(identify_payload)
        .map(|possible_payload| {
          possible_payload.map(|payload| handle_payload(&server_state, payload, &send_tx))
        });

      if last_call.to(PreciseTime::now()).num_seconds() > 2 {
        cull_connections(&mut server_state.users);
      }
    }
  });

  // Block main thread forever
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
      let mut msg: Vec<u8> = message.as_bytes().into_iter().cloned().collect();
      msg.insert(0, message_type_to_byte(MessageType::Message));
      let _ = send_tx.send((server_addr, msg)).unwrap();
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

  // Block main thread forever
  let _ = vec![
    stdout_handle.join(),
    stdin_handle.join(),
    network.thread_handles.send_handle.join(),
    network.thread_handles.recv_handle.join(),
  ];
}
