use std::net::{TcpStream};
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};
use crate::net::{self, parse};
use crate::stats;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

//Reads from TcpStream, writes to File
pub struct FileReceiver {

}

impl FileReceiver {
    pub fn new() -> FileReceiver {
        FileReceiver {}
    }

    pub fn get_file(&mut self, file_name: &str, _port: u16, stream: &mut TcpStream) -> stats::TransferStats {
        let mut buf = [0; net::PACKET_SIZE];
        let mut file = File::create(file_name).expect("File error");

        let mut stats = stats::TransferStats::new();
        let mut realtime_stats = stats::RealtimeStats::new();
        let mut current_bytes = 0;
        loop {
            let _bytes = stream.peek(&mut buf);
            let command = parse::packet(&buf) as u8;

            if command == (net::Code::Data as u8) {
                //println!("\t{}/{}", current_bytes, 000000);
                let bytes = stream.read(&mut buf).unwrap();
                let (_id, _trans, size) = parse::data(buf);
                realtime_stats.set_size(size);

                if current_bytes + bytes >= size {
                    let rem = size - current_bytes;
                    //println!("Current size is {} vs {} {:?} left", current_bytes, size, rem);
                    file.write_all(&buf[net::DATA_OFFSET..(net::DATA_OFFSET + rem)]).expect("Failed to write to stream");
                    realtime_stats.add_bytes(rem);
                    current_bytes += rem;
                    break;
                }
                else {
                    file.write_all(&buf[net::DATA_OFFSET..]).expect("Failed to write to stream");
                }
                current_bytes += bytes - net::DATA_OFFSET;
            }
            else {
                //println!("Exiting with command {}", command);
                break;
            }
        }
        stats.stop(current_bytes);
        stats
    }

    pub fn listen(&mut self, stream: &mut TcpStream) {
        let mut buf = [0; net::PACKET_SIZE];

        loop {
            stream.peek(&mut buf).expect("Unable to peek stream");
            let command = parse::packet(&buf) as u8;

            if command == (net::Code::Redirect as u8) {
                stream.read_exact(&mut buf).unwrap();
                let (port, filename) = parse::redirect(buf);
                let stats = self.get_file(&filename, port, stream);
                println!("{}", stats);
                break;
            }
            else if command == (net::Code::Stdout as u8) {
                stream.read_exact(&mut buf).unwrap();
                let s = String::from_utf8_lossy(&buf[1..]);
                print!("{}", s);
            }
            else if command == (net::Code::End as u8) {
                stream.read_exact(&mut buf).unwrap();
                break;
            }
        }
        println!();
    }

    pub fn delete_file(&self, stream: &mut TcpStream, path: &str) {
        let path = Path::new(path);
        match std::fs::remove_file(path) {
            Ok(_result) => {
                let mut packet = [0; net::PACKET_SIZE];
                packet[0] = net::Code::End as u8;
                stream.write_all(&packet).expect("Unable to write to stream");
            },
            Err(_os) => {
                let mut packet = [0; net::PACKET_SIZE];
                packet[0] = net::Code::Stdout as u8;
                let mut offset = 1;
               
                let message = "Unable to delete file";
                for b in message.as_bytes().iter() {
                    packet[offset] = *b;
                    offset += 1
                }
                stream.write_all(&packet).expect("Unable to write to stream");
                packet[0] = net::Code::End as u8;
                stream.write_all(&packet).expect("Unable to write to stream");
                println!("Unable to delete file"); }
        }
    }
}

//Reads from File, writes to TcpStream
pub struct FileTransmitter {
}

fn get_rate<'a>(bits: usize) -> (f32, &'a str) {
        if bits > 1000000000 {
            (bits as f32 / 1000000000.0, "Gb")
        }
        else if bits > 1000000 {
            (bits as f32 / 1000000.0, "Mb")
        }
        else if bits > 1000 {
            (bits as f32 / 1000.0, "Kb")
        }
        else {
            (bits as f32, "b")
        }
}

impl FileTransmitter {
    pub fn new() -> FileTransmitter {
        FileTransmitter {}
    }

    pub fn host_file(&mut self, path: &str, stream: &mut TcpStream) -> stats::TransferStats {
        let path = Path::new(path).canonicalize().expect("Failed to canonicalize path");
        println!("{:?}", path);

        let mut file = File::open(&path).expect("File Error");
        let size = file.metadata().expect("File Error").len() as u64;

        println!("Hosting file {:?}", &path);

        let mut packet = [0; net::PACKET_SIZE];
        packet[0] = net::Code::Data as u8;
        stream.set_write_timeout(Some(std::time::Duration::new(1, 0))).expect("Unable to set write timeout");
        let progress = ProgressBar::new(size);
        progress.set_style(ProgressStyle::default_spinner()
            .template(" {bytes}/{total_bytes} {wide_msg:.green}")
            .progress_chars("#>-"));

        let mut stats = stats::TransferStats::new();
        let mut realtime_stats = stats::RealtimeStats::new();
        let mut current_bytes: u64 = 0;
        let instant = Instant::now();

        let mut last_second = 0;
        let mut last_bytes = 0;

        loop {
            if instant.elapsed().as_secs() != last_second {
                let bytes = current_bytes - last_bytes;
                let bits = bytes * 8;
                let (bits, rate) = get_rate(bits as usize);

                progress.set_message(&format!("[Transfer Rate: {} {}]", bits, rate));
                progress.inc(1);
                progress.set_position(current_bytes);

                last_second = instant.elapsed().as_secs();
                last_bytes = current_bytes;
            }
            let bytes = file.read(&mut packet[net::DATA_OFFSET..]);
            realtime_stats.set_size(size as usize);

            match bytes {
                Ok(bytes) => {
                    if bytes != 0 {
                        //println!("\t{}/{}", current_bytes, size);
                        net::mod_data(&mut packet, 0x01, current_bytes, size);
                        stream.write_all(&packet).expect("Network error");
                        current_bytes += bytes as u64;
                        realtime_stats.add_bytes(bytes);
                    }
                    else {
                        break;
                    }
                },
                Err(_e) => {}
            }
        }
        stats.stop(current_bytes as usize);
        stats
    }

    pub fn dir(&self, path: &str, stream: &mut TcpStream) {
        let mut packet = [0; net::PACKET_SIZE];
        packet[0] = net::Code::Stdout as u8;
        let mut offset = 1;

        let path = Path::new(path);
        for entry in path.read_dir().expect("Reading directory failed") {
            if let Some(path) = entry.expect("Failed to get entry").path().to_str() {
                    let path  = String::from(path) + "\n";
                    if offset + path.len() >= net::PACKET_SIZE {
                        let fit = path.len() + offset - net::PACKET_SIZE;

                        for b in path.as_bytes().iter().take(fit) {
                            packet[offset] = *b;
                            offset += 1;
                        }

                        stream.write_all(&packet).expect("Network error");

                        offset = 1;
                        for b in path.as_bytes().iter().skip(fit) {
                            packet[offset] = *b;
                            offset += 1;
                        }
                    }
                    else {
                        path.len();
                        for b in path.as_bytes().iter() {
                            packet[offset] = *b;
                            offset += 1
                        }
                    }
            }
        }
        if offset != 1 {
            for b in packet.iter_mut().skip(offset) {
                *b = 0;
            }
        }
        stream.write_all(&packet).expect("Network error");
        packet[0] = net::Code::End as u8;
        stream.write_all(&packet).expect("Network error");
    }
}
