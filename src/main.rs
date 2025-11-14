use std::{env, net::{TcpStream, ToSocketAddrs}, str::FromStr, time::Duration};
use rust_mc_proto::{MinecraftConnection, MCConnTcp, Packet, ProtocolError, prelude::*};
use hostport::HostPort;

const TCP_CONNECTION_TIMEOUT: Duration = Duration::from_millis(5000);

/// Connect to Minecraft Server with TcpStream
pub fn connect_to_minecraft_server(addr: impl ToSocketAddrs) -> Result<MinecraftConnection<TcpStream>, ProtocolError> {
  let addr = match addr.to_socket_addrs() {
    Ok(mut i) => match i.next() {
        Some(i) => i,
        None => return Err(ProtocolError::AddressParseError),
    },
    Err(_) => return Err(ProtocolError::AddressParseError),
  };

  let stream: TcpStream = match TcpStream::connect_timeout(&addr, TCP_CONNECTION_TIMEOUT) {
    Ok(i) => i,
    Err(_) => return Err(ProtocolError::StreamConnectError),
  };

  Ok(MinecraftConnection::new(stream))
}

fn send_handshake(
  conn: &mut MCConnTcp,
  protocol_version: u16,
  server_address: &str,
  server_port: u16,
  next_state: u8,
) -> Result<(), ProtocolError> {
  conn.write_packet(&Packet::build(0x00, |packet| {
    packet.write_varint(protocol_version as i32)?;
    packet.write_string(server_address)?;
    packet.write_unsigned_short(server_port)?;
    packet.write_varint(next_state as i32)
  })?)
}

fn send_status_request(conn: &mut MCConnTcp) -> Result<(), ProtocolError> {
  conn.write_packet(&Packet::empty(0x00))
}

fn read_status_response(conn: &mut MCConnTcp) -> Result<String, ProtocolError> {
  conn.read_packet()?.read_string()
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let input = args.get(1).expect("Address not provided");

  let hostport = HostPort::from_str(input).unwrap_or_else(|_error| {
    HostPort::from_str(&format!("{}:25565", input)).unwrap()
  });

  let mut conn = connect_to_minecraft_server(hostport.to_string()).expect("Couldn't connect to the server...");

  send_handshake(&mut conn, 765, hostport.host(), hostport.port(), 1).unwrap();
  send_status_request(&mut conn).unwrap();

  let json = read_status_response(&mut conn).unwrap();

  println!("{}", json);
}