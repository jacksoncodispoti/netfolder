pub mod server;
pub mod client;

use std::net::{self, TcpStream};
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use std::mem;

pub const PACKET_SIZE: usize = 512;
pub const DATA_OFFSET: usize = 19;

mod error {
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    pub struct ArgError {
        details: String
    }

    impl ArgError {
        pub fn new(msg: &str) -> ArgError {
            ArgError{details: msg.to_string()}
        }
    }

    impl fmt::Display for ArgError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.details)
        }
    }

    impl Error for ArgError {
        fn description(&self) -> &str {
            &self.details
        }
    }
}

pub fn create_redirect(filename: &str, port: u16) -> [u8; PACKET_SIZE] {
    let mut packet = [0; PACKET_SIZE];
    packet[0] = Code::Redirect as u8;
    packet[1] = port as u8;
    packet[2] = (port >> 8) as u8;

    let mut offset = 3;
    for c in filename.as_bytes().iter() {
        packet[offset] = *c;
        offset += 1;
    }

    packet
}

pub fn create_okay() -> [u8; PACKET_SIZE] {
    let mut packet = [0; PACKET_SIZE];
    packet[0] = Code::Okay as u8;
    packet
}

pub fn create_upload(file_name: &str, id: u16) -> [u8; PACKET_SIZE] {
    let mut packet = [0; PACKET_SIZE];
    packet[0] = Code::Upload as u8;
    packet[1] = id as u8;
    packet[2] = (id >> 8) as u8;

    for (i, c) in file_name.as_bytes().iter().enumerate() {
        packet[3 + i] = *c;
    }

    packet
}

pub fn create_download(file_name: &str) -> [u8; PACKET_SIZE] {
    let mut packet = [0; PACKET_SIZE];
    packet[0] = Code::Download as u8;

    let mut offset = 1;
    for c in file_name.as_bytes().iter() {
        packet[offset] = *c;
        offset += 1;
    }

    packet
}



pub fn parse_upload(packet: &[u8; PACKET_SIZE]) -> (String, u16) {
    let b1 = packet[1] as u16;
    let b2 = packet[2] as u16;
    let id = (b2 << 8) | b1;

    let mut name = String::new();
    for c in packet.iter().take(PACKET_SIZE).skip(3) {
        if *c != 0 {
            name.push(*c as char);
        }
        else {
            break;
        }
    }

    (name, id)
}

pub fn parse_download(packet: &[u8; PACKET_SIZE]) -> String {
    String::from(String::from_utf8_lossy(&packet[1..]).into_owned().trim().trim_matches(char::from(0)))
}


pub fn create_error() -> [u8; PACKET_SIZE] {
    let mut packet = [0; PACKET_SIZE];
    packet[0] = Code::Error as u8;
    packet
}

pub fn create_delete(file_name: &str) -> [u8; PACKET_SIZE] {
    let mut packet = [0; PACKET_SIZE];
    packet[0] = Code::Delete as u8;

    for (i, c) in file_name.as_bytes().iter().enumerate() {
        packet[i + 1] = *c;
    }

    packet
}

pub fn create_dir(file_name: &str) -> [u8; PACKET_SIZE] {
    let mut packet = [0; PACKET_SIZE];
    packet[0] = Code::Dir as u8;

    for (i, c) in file_name.as_bytes().iter().enumerate() {
        packet[i + 1] = *c;
    }

    packet
}

pub fn mod_data(packet: &mut [u8; PACKET_SIZE], object: u16, bytes_t: u64, bytes_s: u64) {
    packet[0] = Code::Data as u8;

    packet[1] = object as u8;
    packet[2] = (object >> 8) as u8;

    let mut b1 = [0u8; mem::size_of::<u64>()];
    b1.as_mut()
        .write_u64::<LittleEndian>(bytes_t)
        .expect("Unable to write to packet");

    packet[3..(8 + 3)].clone_from_slice(&b1[..8]);

    let mut b2 = [0u8; mem::size_of::<u64>()];
    b2.as_mut()
        .write_u64::<LittleEndian>(bytes_s)
        .expect("Unable to write to packet");

    packet[11..(8+11)].clone_from_slice(&b2[..8]);
}

pub fn parse_data(packet: [u8; PACKET_SIZE]) -> (u16, u64, usize) {
    let b1 = packet[1] as u16;
    let b2 = packet[2] as u16;
    let id = (b2 << 8) | b1;

    let transmitted = &packet[3..11];
    let total = &packet[11..20];

    let transmitted_u = LittleEndian::read_u64(transmitted);
    let total_u = LittleEndian::read_u64(total);

    (id, transmitted_u, total_u as usize)
}

pub fn parse_redirect(packet: [u8; PACKET_SIZE]) -> (u16, String) {
    let b1 = packet[1] as u16;
    let b2 = packet[2] as u16;

    let file = String::from(String::from_utf8_lossy(&packet[2..]).into_owned().trim().trim_matches(char::from(0)));

    ((b2 << 8) | b1, file)
}

pub fn parse_delete(packet: [u8; PACKET_SIZE]) -> String {
    String::from(String::from_utf8_lossy(&packet[1..]).into_owned().trim().trim_matches(char::from(0)))
}

pub fn parse_dir(_packet: [u8; PACKET_SIZE]) -> String {
    String::new()
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum Code {
    Unknown=0x0,
    Upload=0x1,
    Download=0x2,
    Delete=0x3,
    Dir=0x4,
    Hello=0x5,
    Redirect=0x6,
    Okay=0x7,
    Error=0x8,
    Data=0x9,
    Stdout=0xa,
    End=0xb,
    Disconnect=0xc
}

impl Code {
    pub fn from_u8(value: u8) -> Code {
        match value {
            0x0 => Code::Unknown,
            0x1 => Code::Upload,
            0x2 => Code::Download,
            0x3 => Code::Delete,
            0x4 => Code::Dir,
            0x5 => Code::Hello,
            0x6 => Code::Redirect,
            0x7 => Code::Okay,
            0x8 => Code::Error,
            0x9 => Code::Data,
            0xa => Code::Stdout,
            0xb => Code::End,
            0xc => Code::Disconnect,
            _ => Code::Unknown
        }
    }
}

pub fn parse_packet(packet: &[u8; PACKET_SIZE]) -> Code {
    if packet.is_empty() {
        Code::Unknown
    }
    else {
        Code::from_u8(packet[0])
    }
}

pub struct Connection {
    pub name: String,
    pub stream: Option<TcpStream>
}

impl Connection {
    pub fn new(ip: net::IpAddr, port: u16) -> Connection {
        Connection{name: String::from("Default name"), stream: Some(TcpStream::connect((ip, port)).unwrap())}
    }
    pub fn default() -> Connection {
        Connection{ name: String::from("no connection"), stream: None }
    }

    pub fn connected(&self) -> bool {
        match &self.stream {
            Some(_stream) => true,
            None => false
        }
    }
}
