use clap::{App};

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

    pub fn create_redirect(port: u16) -> [u8; PACKET_SIZE] {
        let mut packet = [0; PACKET_SIZE];
        packet[0] = Code::Redirect as u8;
        packet[1] = port as u8;
        packet[2] = (port >> 8) as u8;
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

    pub fn parse_upload(packet: &[u8; PACKET_SIZE]) -> (String, u16) {
        let b1 = packet[1] as u16;
        let b2 = packet[2] as u16;
        let id = (b2 << 8) | b1;
        
        let mut name = String::new();
        for i in 3..PACKET_SIZE {
            if packet[i] != 0 {
                name.push(packet[i] as char);
            }
            else {
                break;
            }
        }

        (name, id)
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

        for i in 0..8 {
            packet[3 + i] = b1[i];
        }

        let mut b2 = [0u8; mem::size_of::<u64>()];
        b2.as_mut()
            .write_u64::<LittleEndian>(bytes_s)
            .expect("Unable to write to packet");
        for i in 0..8 {
            packet[11 + i] = b2[i];
        }
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

        ((b2 << 8) | b1, String::from("main.rs"))
    }

    pub fn parse_delete(packet: [u8; PACKET_SIZE]) -> String {
        String::new()
    }

    pub fn parse_dir(packet: [u8; PACKET_SIZE]) -> String {
        String::new()
    }

    #[derive(Debug)]
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
        Data=0x9
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
                _ => Code::Unknown
            }
        }
    }

    pub fn parse_packet(packet: &[u8; PACKET_SIZE]) -> Code {
        if packet.len() < 1 {
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

    pub struct ConnectionStream {
        stream: TcpStream
    }

    impl ConnectionStream {
        pub fn new(ip: net::IpAddr, port: u16) -> ConnectionStream {
            let stream = TcpStream::connect((ip, port));
            ConnectionStream{stream: stream.unwrap()}
        }
    }
}
mod encoding {
    use std::net::{TcpStream};
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Write, Seek};
    use crate::net;
    use std::num::Wrapping;

    //Reads from TcpStream, writes to File
    pub struct FileReceiver {

    }

    impl FileReceiver {
        pub fn new() -> FileReceiver {
            FileReceiver {}
        }

        pub fn get_file(&mut self, file_name: &str, port: u16, stream: &mut TcpStream) {
            let mut buf = [0; net::PACKET_SIZE];
            let mut file = File::create(file_name).expect("File error");
            println!("Writing to {}", file_name);

            let mut current_bytes =0;
            loop {
                let bytes = stream.peek(&mut buf);
                let command = net::parse_packet(&buf) as u8;

                if command == (net::Code::Data as u8) {
                    println!("\t{}/{}", current_bytes, 000000);
                    let bytes = stream.read(&mut buf).unwrap();
                    let (id, trans, size) = net::parse_data(buf);

                    if current_bytes + bytes >= size {
                        let rem = size - current_bytes;
                        println!("Current size is {} vs {} {:?} left", current_bytes, size, rem);
                        file.write(&buf[net::DATA_OFFSET..(net::DATA_OFFSET + rem)]);
                        break;
                    }
                    else {
                        file.write(&buf[net::DATA_OFFSET..]);
                    }
                    current_bytes += bytes - net::DATA_OFFSET;
                }
                else {
                    println!("Exiting with command {}", command);
                    break;
                }
            }
            println!("Received file");
        }

        pub fn delete_file(&self, file_name: String) {

        }
    }

    //Reads from File, writes to TcpStream
    pub struct FileTransmitter {
    }

    impl FileTransmitter {
        pub fn new() -> FileTransmitter {
            FileTransmitter {}
        }

        pub fn host_file(&mut self, path: &str, stream: &mut TcpStream) -> u16 {
            let mut file = File::open(path).expect("File Error");
            let size = file.metadata().expect("File Error").len() as u64;

            println!("Hosting file {}", path);

            let mut packet = [0; net::PACKET_SIZE];
            packet[0] = net::Code::Data as u8;
            stream.set_write_timeout(Some(std::time::Duration::new(1, 0)));

            let mut current_bytes: u64 = 0;
            loop {
                let bytes = file.read(&mut packet[net::DATA_OFFSET..]);

                match bytes {
                    Ok(bytes) => {
                        if bytes != 0 {
                            println!("\t{}/{}", current_bytes, size);
                            net::mod_data(&mut packet, 0x01, current_bytes, size);
                            stream.write(&packet).expect("Network error");
                            current_bytes += bytes as u64;
                        }
                        else {
                            break;
                        }
                    },
                    Err(_e) => {}
                }
            }
            0
        }

        pub fn dir(&self, file_name: String) {

        }
    }
}
mod server {
    use std::net::{TcpListener, TcpStream, IpAddr, SocketAddr, Ipv4Addr};
    use std::io::Read;
    use crate::encoding::{FileReceiver, FileTransmitter};
    use crate::net::{self, Code};

    struct Connection {
        stream: TcpStream,
    }

    impl Connection {
        fn new(stream: TcpStream) -> Connection {
            //let encoder = FileEncoder::new(&mut stream);
            Connection { stream: stream }
        }

        fn handle(&mut self) {
            let mut transmitter = FileTransmitter::new();
            let mut receiver = FileReceiver::new();

            let mut buf = [0; net::PACKET_SIZE ];
            loop {
                match self.stream.read(&mut buf) {
                    Ok(size) => {
                        if size != 0 {
                            let code = net::parse_packet(&buf);
                            self.handle_command(&mut transmitter, &mut receiver, code, buf);
                        }
                    },
                    Err(_e) => {}
                }
            }
        }

        fn handle_command(&mut self, transmitter: &mut FileTransmitter, receiver: &mut FileReceiver, command: Code, packet: [u8; net::PACKET_SIZE]) -> [u8; net::PACKET_SIZE] {
            println!("\tGot code {:?}", command);
            match command {
                Code::Upload => {
                    let (name, id) = net::parse_upload(&packet);
                    let res = receiver.get_file(&name, id, &mut self.stream);
                    net::create_okay()
                },
                Code::Delete => {
                    let arg = net::parse_delete(packet);
                    let res = receiver.delete_file(arg);
                    net::create_okay()
                },
                Code::Dir => {
                    let arg = net::parse_dir(packet);
                    let res = transmitter.dir(arg);
                    net::create_okay()
                },
                Code::Redirect => {
                    let (port, filename) = net::parse_redirect(packet);
                    let result = receiver.get_file(&filename, port, &mut self.stream);
                    net::create_okay()
                },
                Code::Download => {
                    let port = transmitter.host_file("NOFILE", &mut self.stream);
                    net::create_redirect(port)
                },
                _ => { println!("Unknown command!"); net::create_error() }
            }
        }
    }

    pub struct NetFolderListener {
        name: String,
        listener: TcpListener
    }

    impl NetFolderListener {
        pub fn new(name: &str, ip: IpAddr, port: u16) -> NetFolderListener {
            let addr = SocketAddr::from((ip, port));
            NetFolderListener{ name: String::from(name), listener: TcpListener::bind(addr).unwrap() }
        }

        pub fn connection_loop(&self) {
            for stream in self.listener.incoming() {
                match stream {
                    Ok(stream) => {
                        println!("Accepted connection from {}", stream.local_addr().unwrap());
                        let mut connection = Connection::new(stream);
                        connection.handle();
                    }
                    Err(e) => {
                        println!("Error accepting incoming connection: {}", e);
                    }
                };
            }
        }

    }

    pub fn start_server() {
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
    use crate::encoding::{FileTransmitter, FileReceiver};
    use colour;

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

    fn upload(transmitter: &mut FileTransmitter, args: Vec<&str>, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            let upload_packet = net::create_upload(args[0], 0x1);
            stream.write(&upload_packet);
            transmitter.host_file(args[0], stream);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }

    fn download(receiver: &mut FileReceiver, args: Vec<&str>, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            receiver.get_file("NOFILE.txt", 0, stream);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }

    fn delete(args: Vec<&str>, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            let delete_packet = net::create_delete(args[0]);
            stream.write(&delete_packet).expect("Network error");
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }

    fn dir(args: Vec<&str>, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 0 {
            let dir_packet = net::create_dir("");
            stream.write(&dir_packet).expect("Network error");
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
            "upload" => { upload(transmitter, args, stream) },
            "download" => { download(receiver, args, stream) },
            "delete" => { delete(args, stream) },
            "dir" => { dir(args, stream) }
            _ => { println!("Connected, Invalid command"); Ok(()) }
        }
    }

    fn parse_command<'a>(command: &'a String) -> (String, Vec<&'a str>) {
        let split = command.trim().split(" ").collect::<Vec<&str>>();

        if split.len() == 0 {
            return (String::new(), split);
        }
        else {
            let mut first = String::from(split[0].trim());
            first.make_ascii_lowercase();

            if split.len() == 1 {
                return (first, Vec::new());
            }
            else {
                return (first, split.into_iter().skip(1).collect::<Vec<&str>>());
            }
        }
    }

    fn client_shell() {
        loop {
            let mut connection = net::Connection::default();
            let mut transmitter = FileTransmitter::new();
            let mut receiver = FileReceiver::new();
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

            let mut stream = connection.stream.expect("This should never happen");

            loop {
                client_prompt("Connected");
                let mut line = String::new();
                io::stdin()
                    .read_line(&mut line)
                    .expect("Failed to read line");

                let (command, args) = parse_command(&line);
                match run_command(&mut transmitter, &mut receiver, &mut stream, &command, args) {
                    Ok(()) => {},
                    Err(e)  => { colour::red_ln!("{:?}", e)}
                }
            }
        }
    }

    pub fn start_client() {
        client_shell();
    }
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
                    .about("Launch a client")
                    .version("0.0.1")
                    .author("Jackson Codispoti <jackson.codispoti@uky.edu>"))
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("server") {
        server::start_server();
        println!("Running the server");
    }
    else if let Some(ref matchef) = matches.subcommand_matches("client") {
        client::start_client();
        println!("Running the client");
    }
    else {
        println!("Please specify server or client");
    }
}
