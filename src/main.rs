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
use net_helpers::get_own_ip;
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

  let local_ip = get_own_ip();
  let server_params = query_server_params();
  let port = server_params.server_port.to_string();
  let full_addr_string: String = local_ip.to_string() + ":" + &port;
  let full_addr: &str = &full_addr_string;
  let (send_tx, send_rx) = channel();
  let (recv_tx, recv_rx) = channel();
  let mut server_state = ServerState { users: HashMap::new() };
  println!("Server Started.");

  let (send_handle, recv_handle) =
    UdpSocket::bind(full_addr)
      .map_err(socket_bind_err)
      .map(|socket| start_network(socket, send_rx, recv_tx))
      .unwrap();

  let chat_handle = thread::spawn (move || {
    loop {
      // TODO: don't broadcast msg we don't care about, rather than send options
      recv_rx.recv()
        .map_err(|err| println!("Recv Err: {}", err))
        .map(|opt| opt.map(|payload| add_user_and_broadcast(&mut server_state, payload, &send_tx)));
    }
  });

  let results = vec![
    chat_handle.join(),
    send_handle.join(),
    recv_handle.join()
  ];
}

fn client() {
  let local_ip = get_own_ip();
  let client_params = query_client_params();
  let client_port = client_params.client_port.to_string();
  let full_client_addr_string: String = local_ip.to_string() + ":" + &client_port;
  let full_client_addr: &str = &full_client_addr_string;
  let full_server_addr_string: String = client_params.server_addr.to_string() + ":" + &client_params.server_port.to_string();
  let server_addr: SocketAddr = SocketAddr::from_str(&full_server_addr_string).unwrap();
  let (send_tx, send_rx) = channel();
  let (recv_tx, recv_rx) = channel();

  let (send_handle, recv_handle) =
    UdpSocket::bind(full_client_addr)
      .map_err(socket_bind_err)
      .map(|socket| start_network(socket, send_rx, recv_tx))
      .unwrap();

  let stdin_handle = thread::spawn (move || {
    let mut stdin = stdin();
    println!("type your messages");
    loop {
      let mut message = String::new();
      stdin.read_line(&mut message);
      send_tx.send((server_addr, message.as_bytes().into_iter().cloned().collect())).unwrap();
    }
  });

  let stdout_handle = thread::spawn (move || {
    loop {
      recv_rx.recv()
        .map_err(|err| println!("Recv Err: {}", err))
        .map(|res| res.map(stringify_body))
        .map(|res| res.map(|(socket_addr, string)| println!("{}", string.trim())));
    }
  });

  let results = vec![
    stdin_handle.join(),
    send_handle.join(),
    recv_handle.join(),
  ];
}
