use clap::{App, Arg};

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
mod net {
    use std::net::{self, TcpStream};
    use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
    use std::mem;

    pub const PACKET_SIZE: usize = 512;
    pub const DATA_OFFSET: usize = 19;

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
}
mod stats {
    use std::time::Instant;
    use std::fmt;

    #[derive(Debug)]
    pub struct TransferStats {
        elapsed: f32,
        bytes: usize,
        instant: Instant 
    }

    impl fmt::Display for TransferStats {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            //Use Kb/s and stuff
            let bits = self.bytes * 8;

            let (rate, rate_name) = 
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
            };

            write!(f, "{} {}/s", rate, rate_name)
        }
    }

    impl TransferStats {
        pub fn new() -> TransferStats {
            TransferStats { elapsed: 0.0, bytes: 0, instant: Instant::now() }
        }

        //pub fn start(&mut self) {
        //    self.instant = Instant::now();
        //}

        pub fn stop(&mut self, bytes: usize) {
            self.elapsed = self.instant.elapsed().as_nanos() as f32 / 1000.0;
            self.bytes = bytes;
        }
    }

    #[derive(Debug)]
    pub struct RealtimeStats {
        instant: Instant,
        current_bytes: usize,
        size: usize,
        measures: Vec<(u64, usize)>
    }

    impl RealtimeStats {
        pub fn new() -> RealtimeStats {
            RealtimeStats { instant: Instant::now(), current_bytes: 0, size: 0, measures: vec![(0, 0)] }
        }

        pub fn set_size(&mut self, size: usize) {
            if self.size == 0 {
                self.size = size
            }
        }
        pub fn add_bytes(&mut self, bytes: usize) {
            self.measures.push((self.instant.elapsed().as_nanos() as u64, self.current_bytes));
            self.current_bytes += bytes;
        }

        // Return speed in bits
        pub fn get_speed(&mut self) -> (usize, u64) {
            // Look at past second
            const SECOND: u64 = 1000000000;

            let (last_time, last_bytes) = match self.measures.last() {
               Some((last_time, last_bytes)) => (*last_time, *last_bytes),
               None => (0, 0)
            }; 

            if (last_time, last_bytes) != (0, 0) {
                let threshold = if last_time > SECOND { last_time - SECOND } else { 0 };

                let mut c = 0;
                for m in self.measures.iter() {
                    if m.0 < threshold {
                        c += 1;
                    }
                    else {
                        break;
                    }
                }

                for i in 0..c {
                    self.measures.remove(0);
                }

                if let Some((start_time, start_bytes)) = self.measures.first() {
                    (last_bytes - start_bytes, last_time - start_time)
                }
                else {
                   (self.current_bytes - last_bytes, SECOND)
                }

            }
            else {
                println!("nope");
               (0, SECOND)
            }
        }
    }
}
mod encoding {
    use std::net::{TcpStream};
    use std::fs::File;
    use std::path::Path;
    use std::io::{Read, Write};
    use crate::net;
    use crate::stats;
    use indicatif::{ProgressBar, ProgressStyle};

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
                let command = net::parse_packet(&buf) as u8;

                if command == (net::Code::Data as u8) {
                    //println!("\t{}/{}", current_bytes, 000000);
                    let bytes = stream.read(&mut buf).unwrap();
                    let (_id, _trans, size) = net::parse_data(buf);
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
                let command = net::parse_packet(&buf) as u8;

                if command == (net::Code::Redirect as u8) {
                    stream.read_exact(&mut buf).unwrap();
                    let (port, filename) = net::parse_redirect(buf);
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
            let mut current_packet = 0;
            loop {
                current_packet += 1;
                if (current_packet % 200) == 1 {
                            let (bytes, time) = realtime_stats.get_speed();
                            let bits = bytes * 8;
                            let (bits, rate) = get_rate(bits);

                            progress.set_message(&format!("[Transfer Rate: {} {}]", bits, rate));
                            progress.inc(1);
                            progress.set_position(current_bytes);
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
}
mod server {
    use std::net::{TcpListener, TcpStream, IpAddr, SocketAddr, Ipv4Addr};
    use std::io::{Read, Write};
    use crate::encoding::{FileReceiver, FileTransmitter};
    use crate::net::{self, Code};
    use std::thread;

    struct Connection {
        stream: TcpStream,
    }

    impl Connection {
        fn new(stream: TcpStream) -> Connection {
            //let encoder = FileEncoder::new(&mut stream);
            Connection { stream }
        }

        fn handle(&mut self) {
            let addr = self.stream.local_addr().unwrap();
            let mut transmitter = FileTransmitter::new();
            let mut receiver = FileReceiver::new();

            let mut buf = [0; net::PACKET_SIZE ];
            println!("[{}] Connection initiated", addr);
            loop {
                match self.stream.read(&mut buf) {
                    Ok(size) => {
                        if size != 0 {
                            let code = net::parse_packet(&buf);

                            if code == net::Code::Disconnect {
                                break;
                            }
                            self.handle_command(&mut transmitter, &mut receiver, code, buf, &addr);
                        }
                    },
                    Err(_e) => {}
                }
            }
            println!("[{}] Connection ended", addr);
        }

        fn handle_command(&mut self, transmitter: &mut FileTransmitter, receiver: &mut FileReceiver, command: Code, packet: [u8; net::PACKET_SIZE], addr: &std::net::SocketAddr) -> [u8; net::PACKET_SIZE] {
            //println!("[{}] Received code {:?}", addr, command);
            match command {
                Code::Upload => {
                    let (name, id) = net::parse_upload(&packet);
                    println!("[{}] Receiving upload: {}", addr, name);
                    let stats = receiver.get_file(&name, id, &mut self.stream);
                    println!("[{}]\t{}: {}", addr, name, stats);
                    net::create_okay()
                },
                Code::Delete => {
                    let arg = net::parse_delete(packet);
                    let _res = receiver.delete_file(&mut self.stream, &arg);
                    net::create_okay()
                },
                Code::Dir => {
                    let _arg = net::parse_dir(packet);
                    let _res = transmitter.dir("./", &mut self.stream);
                    net::create_okay()
                },
                Code::Redirect => {
                    let (port, filename) = net::parse_redirect(packet);
                    let stats = receiver.get_file(&filename, port, &mut self.stream);
                    println!("[{}] {}", addr, stats);
                    net::create_okay()
                },
                Code::Download => {
                    let path = net::parse_download(&packet);
                    self.stream.write_all(&net::create_redirect(&path, 0)).expect("Network error");
                    let _stats = transmitter.host_file(&path, &mut self.stream);
                    net::create_okay()
                },
                _ => { println!("[{}] Unknown command!", addr); net::create_error() }
            }
        }
    }

    pub struct NetFolderListener {
        _name: String,
        listener: TcpListener
    }

    impl NetFolderListener {
        pub fn new(name: &str, ip: IpAddr, port: u16) -> NetFolderListener {
            let addr = SocketAddr::from((ip, port));
            NetFolderListener{ _name: String::from(name), listener: TcpListener::bind(addr).unwrap() }
        }

        pub fn connection_loop(&self) {
            for stream in self.listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let mut connection = Connection::new(stream);
                        //connection.handle();
                        thread::spawn(move || { connection.handle() });
                    }
                    Err(e) => {
                        println!("Error accepting incoming connection: {}", e);
                    }
                };
            }
        }
    }

    pub fn start_server(_matches: &clap::ArgMatches) {
        let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        let port = 3219;
        let listener = NetFolderListener::new("TheBlackPearl", ip, port);

        listener.connection_loop();
    }
}

mod client {
    use std::io::{self, Write};
    use std::net::{TcpStream};
    use crate::net;
    use std::error::Error;
    use crate::error;
    use std::path::Path;
    use crate::encoding::{FileTransmitter, FileReceiver};

    fn client_prompt(name: &str) {
        print!("({}) > ", name);
        io::stdout().flush().unwrap();
    }

    fn connect(args: Vec<&str>, connection: &mut net::Connection) -> Result<(), Box<dyn Error>> {
        if args.len() == 2 {
            let ip: std::net::IpAddr = args[0].parse().unwrap();
            let port: u16 = args[1].parse().unwrap();

            *connection = net::Connection::new(ip, port);

            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 2 arguments")))
        }
    }

    fn upload(transmitter: &mut FileTransmitter, stream: &mut TcpStream, path: &Path) {
        let upload_packet = net::create_upload(path.file_name().unwrap().to_str().unwrap(), 0x1);
        stream.write_all(&upload_packet).expect("Unable to write to stream");
        let stats = transmitter.host_file(path.to_str().unwrap(), stream);
        println!("{}", stats);
    }
    fn shell_upload(transmitter: &mut FileTransmitter, args: Vec<&str>, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            let path = Path::new(args[0]);
            upload(transmitter, stream, &path);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }

    fn download(receiver: &mut FileReceiver, stream: &mut TcpStream, path: &str) {
        let download_packet = net::create_download(path);
        stream.write_all(&download_packet).expect("Unable to write to stream");
        receiver.listen(stream);
    }
    fn shell_download(receiver: &mut FileReceiver, args: Vec<&str>, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            download(receiver, stream, args[0]);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }

    fn delete(receiver: &mut FileReceiver, stream: &mut TcpStream, path: &str) {
            let delete_packet = net::create_delete(path);
            stream.write_all(&delete_packet).expect("Network error");
            receiver.listen(stream);
    }
    fn shell_delete(args: Vec<&str>, receiver: &mut FileReceiver, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            delete(receiver, stream, args[0]);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }

    fn dir(receiver: &mut FileReceiver, stream: &mut TcpStream) {
        let dir_packet = net::create_dir("");
        stream.write_all(&dir_packet).expect("Network error");
        receiver.listen(stream);
    }
    fn shell_dir(args: Vec<&str>, receiver: &mut FileReceiver, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.is_empty() {
            dir(receiver, stream);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 0 arguments")))
        }
    }

    fn pre_run_command(connection: &mut net::Connection, command: &str, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
        match command {
            "connect" => { connect(args, connection) },
            _ => { println!("Not connected, invalid command"); Ok(()) }
        }
    }

    fn run_command(transmitter: &mut FileTransmitter, receiver: &mut FileReceiver, stream: &mut TcpStream, command: &str, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
        match command {
            "upload" => { shell_upload(transmitter, args, stream) },
            "download" => { shell_download(receiver, args, stream) },
            "delete" => { shell_delete(args, receiver, stream) },
            "dir" => { shell_dir(args, receiver, stream) }
            _ => { println!("Connected, Invalid command"); Ok(()) }
        }
    }

    fn parse_command(command: &'_ str) -> (String, Vec<&'_ str>) {
        let split = command.trim().split(' ').collect::<Vec<&str>>();

        if split.is_empty() {
            (String::new(), split)
        }
        else {
            let mut first = String::from(split[0].trim());
            first.make_ascii_lowercase();

            if split.len() == 1 {
                (first, Vec::new())
            }
            else {
                (first, split.into_iter().skip(1).collect::<Vec<&str>>())
            }
        }
    }

    fn open_connection(ip_str: &str, port: u16) -> net::Connection {
        let ip: std::net::IpAddr = ip_str.parse().unwrap();

        net::Connection::new(ip, port)
    }

    fn disconnect(stream: &mut TcpStream, _transmitter: &mut FileTransmitter, _receiver: &mut FileReceiver) {
        let mut packet = [0; net::PACKET_SIZE];
        packet[0] = net::Code::Disconnect as u8;
        stream.write_all(&packet).expect("Network error");
    }

    fn pre_connection_shell() -> net::Connection {
        let mut connection = net::Connection::default();
        while !connection.connected() {
            client_prompt("not-connected");
            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");

            let (command, args) = parse_command(&line);
            match pre_run_command(&mut connection, &command, args) {
                Ok(()) => {},
                Err(e)  => { colour::red_ln!("{:?}", e)}
            }
        }

        connection
    }
    fn post_connection_shell(stream: TcpStream, transmitter: FileTransmitter, receiver: FileReceiver) {
        let mut stream  = stream;
        let mut transmitter = transmitter;
        let mut receiver = receiver;

        loop {
            client_prompt("Connected");
            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");

            let (command, args) = parse_command(&line);
            if command == "exit" {
                break;
            }
            match run_command(&mut transmitter, &mut receiver, &mut stream, &command, args) {
                Ok(()) => {},
                Err(e)  => { colour::red_ln!("{:?}", e)}
            }
        }
        disconnect(&mut stream, &mut transmitter, &mut receiver);
    }

    pub fn start_client(matches: &clap::ArgMatches) {
        let connection = if matches.is_present("host") && matches.is_present("port") {
            let host = matches.value_of("host").unwrap();
            let port: u16 = matches.value_of("port").unwrap().parse().expect("Please provide a valid port");
            open_connection(host, port)
        }
        else {
            pre_connection_shell()
        };
        let mut transmitter = FileTransmitter::new();
        let mut receiver = FileReceiver::new();
        let mut stream = connection.stream.expect("This should never happen");

        let mut had_cmd = false;

        if matches.is_present("list") {
            dir(&mut receiver, &mut stream); 
            had_cmd = true;
        }

        if matches.is_present("download") {
            let path = matches.value_of("download").unwrap();
            download(&mut receiver, &mut stream, path); 
            had_cmd = true;
        }

        if matches.is_present("upload") {
            let path = Path::new(matches.value_of("upload").unwrap());
            upload(&mut transmitter, &mut stream, &path); 
            had_cmd = true;
        }

        if matches.is_present("delete") {
            let path = matches.value_of("delete").unwrap();
            delete(&mut receiver, &mut stream, path); 
            had_cmd = true;
        }

        if matches.is_present("shell") || !had_cmd {
            post_connection_shell(stream, transmitter, receiver);
        }
        else {
            disconnect(&mut stream, &mut transmitter, &mut receiver);
        }
    }
}

macro_rules! arg {
    ($t:expr) => {
        Arg::new($t).long($t);
    };
}

fn main() {
    let matches = App::new("Net-Folder")
        .version("0.0.1")
        .author("Jackson Codispoti <jackson.codispoti@uky.edu>")
        .about("Connect to another PC and transfer files")
        .subcommand(App::new("server")
                    .about("Launch a server")
                    .version("0.0.1")
                    .author("Jackson Codispoti <jackson.codispoti@uky.edu>"))
        .subcommand(App::new("client")
                    .arg(arg!("host")
                         .short('n')
                         .takes_value(true)
                         .about("The hostname to connect to"))
                    .arg(arg!("port")
                         .short('p')
                         .takes_value(true)
                         .about("The port to connect to"))

                    .arg(arg!("upload")
                         .short('u')
                         .takes_value(true)
                         .about("The file to upload"))
                    .arg(arg!("download")
                         .short('d')
                         .takes_value(true)
                         .about("The file to download"))
                    .arg(arg!("delete")
                         .short('D')
                         .takes_value(true)
                         .about("The file to delete"))
                    .arg(arg!("list")
                         .short('l')
                         .takes_value(false)
                         .about("List files on server"))

                    .arg(Arg::new("shell")
                         .long("shell")
                         .short('s')
                         .takes_value(false)
                         .about("Forces to start in shell mode. Undefined behaviours"))
                    .about("Launch a client")
                    .version("0.0.1")
                    .author("Jackson Codispoti <jackson.codispoti@uky.edu>"))
                    .get_matches();

    if let Some(server_matches) = matches.subcommand_matches("server") {
        server::start_server(server_matches);
        //println!("Running the server");
    }
    else if let Some(client_matches) = matches.subcommand_matches("client") {
        client::start_client(client_matches);
        //println!("Running the client");
    }
    else {
        println!("Please specify server or client");
    }
}
