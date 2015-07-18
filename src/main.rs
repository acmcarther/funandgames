#![feature(lookup_host)]
#![feature(convert)]
#![feature(core)]
use std::env;
use std::ascii::AsciiExt;
use std::io::stdin;
use std::net::{SocketAddr, UdpSocket};
use std::io::Error;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::collections::HashMap;
use std::thread::JoinHandle;
use std::str::FromStr;

mod str_ops;
mod params;
mod constants;
mod errors;
use params::{
  query_server_params,
  query_client_params,
  ClientParams,
  ServerParams,
};
use constants::{LOCAL_IP, UDP_MARKER};
use errors::{socket_bind_err, socket_send_err, socket_recv_err};

#[derive(Debug)]
enum NetMode {
  Client,
  Server
}

struct ServerState {
  users: HashMap<String, SocketAddr>
}

type RawSocketPayload = (SocketAddr, Vec<u8>);
type SocketPayload    = (SocketAddr, Vec<u8>);
type StringPayload    = (SocketAddr, String);

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
  let full_addr_string: String = LOCAL_IP.to_string() + ":" + &port;
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
        .map(|opt| opt.map(|payload| add_and_broadcast(&mut server_state, payload, &send_tx)));
    }
  });

  chat_handle.join();
  send_handle.join();
  recv_handle.join();
}

fn client() {
  let client_params = query_client_params();
  let client_port = client_params.client_port.to_string();
  let full_client_addr_string: String = LOCAL_IP.to_string() + ":" + &client_port;
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

  stdin_handle.join();
  send_handle.join();
  recv_handle.join();
}


fn start_network(
  socket: UdpSocket,
  send_rx: Receiver<SocketPayload>,
  recv_tx: Sender<Result<SocketPayload, Error>>,
  ) -> (JoinHandle< ()>, JoinHandle<()>) {

  let recv_socket = socket.try_clone().unwrap();

  // Sending messages
  let send_handle = thread::spawn (move || {
    loop {
      send_rx.recv()
        .map(add_payload_marker)
        .map(|(socket_addr, payload)| socket.send_to(payload.as_slice(), socket_addr));
    }
  });

  // Receiving messages
  let recv_handle = thread::spawn (move || {
    loop {
      let mut buf = [0; 256];
      let result = recv_socket.recv_from(&mut buf)
        .map(|(_, socket_addr)| (socket_addr, buf.to_vec()))
        .map(starts_with_marker)
        .map(|payload| payload.map(strip_marker));

      // Invert the monads, send if Some(res) or Err
      match result {
        Ok(opt) => opt.map (|val| Ok(val)),
        Err(err) => Some(Err(err))
      }.map(|res| recv_tx.send(res));
    }
  });

  (send_handle, recv_handle)
}

fn net_mode_from_string(mode_str: &str) -> NetMode {
  match mode_str.to_ascii_lowercase().as_ref() {
    "server" => NetMode::Server,
    _ => NetMode::Client
  }
}

fn starts_with_marker(payload: RawSocketPayload) -> Option<RawSocketPayload> {
  let (socketAddr, payload) = payload;
  if &payload[0..3] == UDP_MARKER {
    Some((socketAddr, payload))
  } else {
    None
  }
}

fn add_payload_marker(payload: SocketPayload) -> RawSocketPayload {
  let (socket_addr, payload) = payload;
  let mut buf = [0; 256];
  let marked_bytes: Vec<u8> = UDP_MARKER.into_iter().cloned().chain(payload.iter().cloned()).collect();
  (socket_addr, marked_bytes)
}

fn strip_marker(payload: RawSocketPayload) -> SocketPayload {
  let (socket_addr, payload) = payload;
  (socket_addr, payload[3..256].into_iter().cloned().collect())
}

fn add_and_broadcast(server_state: &mut ServerState, payload: SocketPayload, send_tx: &Sender<SocketPayload>) {
  let (socket_addr, _) = payload;
  add_user(server_state, socket_addr);
  reply_all(server_state, payload, send_tx);
}

fn add_user(server_state: &mut ServerState, socket_addr: SocketAddr) {
  let key = socket_addr.to_string();
  if !server_state.users.contains_key(&key) {
    println!("saving user");
    server_state.users.insert(key, socket_addr.clone());
  }
}

fn reply_all(server_state: &ServerState, payload: SocketPayload, send_tx: &Sender<SocketPayload>) {
  let (from_socket_addr, payload_bytes) = payload;
  let from_socket_addr_str = from_socket_addr.to_string();
  let name_with_colon = from_socket_addr_str.clone() + ": ";
  let name_bytes = name_with_colon.as_bytes();
  let full_payload_bytes: Vec<u8> = name_bytes.into_iter().cloned().chain(payload_bytes.iter().cloned()).collect();
  server_state.users.values().map (|socket_addr| {
    if from_socket_addr_str != socket_addr.to_string() {
      send_tx.send((socket_addr.clone(), full_payload_bytes.clone()));
    }
  }).collect::<Vec<()>>();
}

fn stringify_body(payload: SocketPayload) -> StringPayload {
  let (socket_addr, bytes) = payload;
  let full_msg = String::from_utf8(bytes).unwrap();
  (socket_addr, full_msg.trim_matches('\0').to_string())
}
