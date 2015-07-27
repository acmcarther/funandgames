#![feature(lookup_host)]
#![feature(ip_addr)]
use std::env;
use std::io::stdin;
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};

use time::PreciseTime;

extern crate time;
extern crate tap;
extern crate game_udp;
extern crate byteorder;

mod app_net;
mod connected_udp;
mod net_helpers;
mod params;
mod str_ops;
mod types;

use app_net::{
  identify_payload,
  handle_payload,
  stringify_body
};
use params::{
  query_server_params,
  query_client_params,
};
use types::{message_type_to_byte, MessageType, NetMode, ServerState};
use str_ops::{net_mode_from_string};
use game_udp::packet_types::Packet;
use game_udp::start_network;
use connected_udp::{handle_connections, cull_connections};
use tap::Tappable;

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

  let send_tx: Sender<Packet> = network.send_channel;
  let recv_rx: Receiver<Packet> = network.recv_channel;

  let chat_handle = thread::spawn (move || {
    let mut last_heartbeat = PreciseTime::now();
    loop {
      let last_call = PreciseTime::now();

      let _ = recv_rx.try_recv().ok()
        .tap(|payload| handle_connections(payload, &mut server_state.users))
        .map(identify_payload)
        .map(|possible_payload| {
          possible_payload.map(|payload| handle_payload(&server_state, payload, &send_tx))
        });

      if last_call.to(PreciseTime::now()).num_seconds() > 2 {
        cull_connections(&mut server_state.users);
      }

      if last_heartbeat.to(PreciseTime::now()).num_seconds() >= 1 {
        server_state.users.keys().map (|socket_addr| {
          let _ = send_tx.send(Packet{addr: socket_addr.clone(), bytes: vec![message_type_to_byte(MessageType::KeepAlive)]}).unwrap();
        }).collect::<Vec<()>>();
        last_heartbeat = PreciseTime::now();
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

  let send_tx: Sender<Packet> = network.send_channel;
  let recv_rx: Receiver<Packet> = network.recv_channel;
  let (stdin_tx, stdin_rx) = channel();

  let stdin_handle = thread::spawn (move || {
    let mut stdin = stdin();
    println!("type your messages");
    loop {
      let mut message = String::new();
      let _ = stdin.read_line(&mut message);
      let _ = stdin_tx.send(message);
    }
  });

  let send_handle = thread::spawn (move || {
    let mut last_heartbeat = PreciseTime::now();
    loop {
      thread::sleep_ms(100);
      let possible_chat = stdin_rx.try_recv().ok();
      possible_chat.map (|message| {
        let mut msg: Vec<u8> = message.as_bytes().into_iter().cloned().collect();
        msg.insert(0, message_type_to_byte(MessageType::Message));
        let _ = send_tx.send(Packet{addr: server_addr, bytes: msg}).unwrap();
      });
      if last_heartbeat.to(PreciseTime::now()).num_seconds() >= 1 {
        let _ = send_tx.send(Packet{addr: server_addr, bytes: vec![message_type_to_byte(MessageType::KeepAlive)]}).unwrap();
        last_heartbeat = PreciseTime::now();
      }
    }
  });

  let stdout_handle = thread::spawn (move || {
    loop {
      let _ = recv_rx.recv().ok()
        .and_then(identify_payload)
        .and_then(|possible_payload| {
          match possible_payload.msg_type {
            MessageType::Message => Some(stringify_body(possible_payload.bytes)),
            _ => None
          }
        })
        .map(|msg| println!("{}", msg.trim()));
    }
  });

  // Block main thread forever
  let _ = vec![
    stdout_handle.join(),
    stdin_handle.join(),
    send_handle.join(),
    network.thread_handles.send_handle.join(),
    network.thread_handles.recv_handle.join(),
  ];
}
