use std::{env, net::{TcpStream, ToSocketAddrs}, process::exit, str::FromStr, time::Duration};
use resolve::{DnsConfig, DnsResolver, record::Srv};
use rust_mc_proto::{MinecraftConnection, MCConnTcp, Packet, ProtocolError, prelude::*};
use hostport::HostPort;

const TCP_CONNECTION_TIMEOUT: Duration = Duration::from_millis(5000);

const ERROR_CODE_GENERAL: i32 = 1;
const ERROR_CODE_ADDRESS: i32 = 2;
const ERROR_CODE_STREAM: i32 = 3;

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

fn lookup_srv_record(address: &String) -> Option<(String, u16)> {
  let config = match DnsConfig::load_default() {
    Ok(config) => config,
    Err(_) => return None,
  };

  let resolver = match DnsResolver::new(config) {
    Ok(resolver) => resolver,
    Err(_) => return None,
  };

  let name = format!("_minecraft._tcp.{address}");

  match resolver.resolve_record::<Srv>(&name) {
    Ok(records) => {
      if let Some(record)= records.get(0) {
        Some((record.target.to_string(), record.port))
      } else {
        None
      }
    },
    Err(_) => None,
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let input = args.get(1).unwrap_or_else(|| {
    eprintln!("No address provided.");
    exit(ERROR_CODE_GENERAL);
  });
  
  let host: String;
  let port: u16;

  if input.contains(':') {
    let hostport = HostPort::from_str(input).unwrap_or_else(|_error| {
      HostPort::from_str(&format!("{}:25565", input)).unwrap_or_else(|_error2| {
        eprintln!("Address could not be parsed.");
        exit(ERROR_CODE_GENERAL);      
      })
    });

    host = hostport.host().to_string();
    port = hostport.port();
  } else {
    if let Some((srv_host, srv_port)) = lookup_srv_record(input) {
      host = srv_host;
      port = srv_port;
    } else {
      host = input.to_string();
      port = 25565;
    }
  }

  let mut conn = connect_to_minecraft_server(format!("{host}:{port}")).unwrap_or_else(|error| {
      match error {
        ProtocolError::AddressParseError => {
          eprintln!("Address could not be parsed.");
          exit(ERROR_CODE_ADDRESS)
        },
        ProtocolError::StreamConnectError => {
          eprintln!("Stream connection error.");
          exit(ERROR_CODE_STREAM)
        },
        _ => todo!(),
      }
    }
  );

  send_handshake(&mut conn, 765, &host, port, 1).unwrap();
  send_status_request(&mut conn).unwrap();

  let json = read_status_response(&mut conn).unwrap();

  println!("{}", json);
}