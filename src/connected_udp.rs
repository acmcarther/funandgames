pub use self::connected_udp::{
  ConnectionTable,
  Connection,
  handle_connections,
  cull_connections
};

mod connected_udp {
  use std::net::SocketAddr;
  use std::collections::HashMap;
  use time::PreciseTime;

  use types::SocketPayload;

  pub type ConnectionTable = HashMap<SocketAddr, Connection>;

  pub struct Connection {
    pub last_contact: PreciseTime
  }

  pub fn handle_connections(payload: SocketPayload, connections: &mut ConnectionTable) -> SocketPayload {
    let (socket_addr, _) = payload;

    let _ =
      connections.insert(socket_addr.clone(), Connection { last_contact: PreciseTime::now() });

    payload
  }

  pub fn cull_connections(connections: &mut ConnectionTable) {
    let now = PreciseTime::now();
    let stale_connections: Vec<SocketAddr> =
      connections.iter()
        .filter(|&(_, connection)| connection.last_contact.to(now).num_seconds() > 5 )
        .map (|(&socket_addr, _)| socket_addr.clone())
        .collect();

    for socket_addr in stale_connections {
      println!("culling connection: {}", socket_addr);
      connections.remove(&socket_addr);
    }
  }
}
