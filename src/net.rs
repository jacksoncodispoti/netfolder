pub mod server;
pub mod client;

use std::net::{self, TcpStream};
use std::mem;

pub const PACKET_SIZE: usize = 512;
pub const DATA_OFFSET: usize = 19;

#[derive(Debug)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub enum Code {
    Unknown=0x0,
    Upload=0x1,
    Download=0x2,
    Delete=0x3,
    Dir=0x4,
    Redirect=0x6,
    Okay=0x7,
    Error=0x8,
    Data=0x9,
    Stdout=0xa,
    End=0xb,
    Disconnect=0xc
}

impl Code {
    pub fn packet(&self) -> [u8; PACKET_SIZE] {
        let mut packet = [0; PACKET_SIZE];
        packet[0] = *self as u8;
        packet
    }

    pub fn from_u8(value: u8) -> Code {
        match value {
            0x0 => Code::Unknown,
            0x1 => Code::Upload,
            0x2 => Code::Download,
            0x3 => Code::Delete,
            0x4 => Code::Dir,
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

pub mod create {
    use crate::net::{Code, PACKET_SIZE};

    pub fn upload(file_name: &str, id: u16) -> [u8; PACKET_SIZE] {
        let mut packet = Code::Upload.packet();
        packet[1] = id as u8;
        packet[2] = (id >> 8) as u8;

        for (i, c) in file_name.as_bytes().iter().enumerate() {
            packet[3 + i] = *c;
        }

        packet
    }

    pub fn download(file_name: &str) -> [u8; PACKET_SIZE] {
        let mut packet = Code::Download.packet();

        let mut offset = 1;
        for c in file_name.as_bytes().iter() {
            packet[offset] = *c;
            offset += 1;
        }

        packet
    }

    pub fn delete(file_name: &str) -> [u8; PACKET_SIZE] {
        let mut packet = Code::Delete.packet();

        for (i, c) in file_name.as_bytes().iter().enumerate() {
            packet[i + 1] = *c;
        }

        packet
    }

    pub fn dir(file_name: &str) -> [u8; PACKET_SIZE] {
        let mut packet = Code::Dir.packet();

        for (i, c) in file_name.as_bytes().iter().enumerate() {
            packet[i + 1] = *c;
        }

        packet
    }

    pub fn redirect(filename: &str, port: u16) -> [u8; PACKET_SIZE] {
        let mut packet = Code::Redirect.packet();
        packet[1] = port as u8;
        packet[2] = (port >> 8) as u8;

        let mut offset = 3;
        for c in filename.as_bytes().iter() {
            packet[offset] = *c;
            offset += 1;
        }

        packet
    }
}

pub mod parse {
    use byteorder::{ByteOrder, LittleEndian};
    use crate::net::{Code, PACKET_SIZE};

    pub fn packet(packet: &[u8; PACKET_SIZE]) -> Code {
        if packet.is_empty() {
            Code::Unknown
        }
        else {
            Code::from_u8(packet[0])
        }
    }

    pub fn upload(packet: &[u8; PACKET_SIZE]) -> (String, u16) {
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

    pub fn download(packet: &[u8; PACKET_SIZE]) -> String {
        String::from(String::from_utf8_lossy(&packet[1..]).into_owned().trim().trim_matches(char::from(0)))
    }

    pub fn delete(packet: [u8; PACKET_SIZE]) -> String {
        String::from(String::from_utf8_lossy(&packet[1..]).into_owned().trim().trim_matches(char::from(0)))
    }

    pub fn dir(_packet: [u8; PACKET_SIZE]) -> String {
        String::new()
    }

    pub fn redirect(packet: [u8; PACKET_SIZE]) -> (u16, String) {
        let b1 = packet[1] as u16;
        let b2 = packet[2] as u16;

        let file = String::from(String::from_utf8_lossy(&packet[2..]).into_owned().trim().trim_matches(char::from(0)));

        ((b2 << 8) | b1, file)
    }

    pub fn data(packet: [u8; PACKET_SIZE]) -> (u16, u64, usize) {
        let b1 = packet[1] as u16;
        let b2 = packet[2] as u16;
        let id = (b2 << 8) | b1;

        let transmitted = &packet[3..11];
        let total = &packet[11..20];

        let transmitted_u = LittleEndian::read_u64(transmitted);
        let total_u = LittleEndian::read_u64(total);

        (id, transmitted_u, total_u as usize)
    }
}

use byteorder::{LittleEndian, WriteBytesExt};

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
