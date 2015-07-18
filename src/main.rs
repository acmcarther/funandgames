#![feature(convert)]
#![feature(lookup_host)]
#![feature(ip_addr)]
use std::env;
use std::io::stdin;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::channel;
use std::thread;
use std::collections::HashMap;
use std::str::FromStr;

mod app_net;
mod errors;
mod net_helpers;
mod params;
mod str_ops;
mod types;
mod udp;

use app_net::{add_user_and_broadcast, stringify_body};
use errors::{socket_bind_err};
use params::{
  query_server_params,
  query_client_params,
};
use types::{NetMode, ServerState};
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
  let (send_tx, send_rx) = channel();
  let (recv_tx, recv_rx) = channel();
  let mut server_state = ServerState { users: HashMap::new() };
  println!("Server Started on {}", server_params.addr);

  let (send_handle, recv_handle) =
    UdpSocket::bind(server_params.addr)
      .map_err(socket_bind_err)
      .map(|socket| start_network(socket, send_rx, recv_tx))
      .unwrap();

  let chat_handle = thread::spawn (move || {
    loop {
      // TODO: don't broadcast msg we don't care about, rather than send options
      let _ = recv_rx.recv()
        .map_err(|err| println!("Recv Err: {}", err))
        .map(|opt| opt.map(|payload| add_user_and_broadcast(&mut server_state, payload, &send_tx)));
    }
  });

  let _ = vec![
    chat_handle.join(),
    send_handle.join(),
    recv_handle.join()
  ];
}

fn client() {
  let client_params = query_client_params();
  let server_addr = client_params.server_addr;
  let addr = client_params.addr;
  let (send_tx, send_rx) = channel();
  let (recv_tx, recv_rx) = channel();
  println!("Client Started on {}", client_params.addr);
  println!("     to server on {}", client_params.server_addr);

  let (send_handle, recv_handle) =
    UdpSocket::bind(addr)
      .map_err(socket_bind_err)
      .map(|socket| start_network(socket, send_rx, recv_tx))
      .unwrap();

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
      let _ = recv_rx.recv()
        .map_err(|err| println!("Recv Err: {}", err))
        .map(|res| res.map(stringify_body))
        .map(|res| res.map(|(_, string)| println!("{}", string.trim())));
    }
  });

  let _ = vec![
    stdout_handle.join(),
    stdin_handle.join(),
    send_handle.join(),
    recv_handle.join(),
  ];
}
