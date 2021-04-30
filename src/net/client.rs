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

mod commands {
    use std::net::{IpAddr, TcpStream};
    use std::path::Path;
    use std::io::Write;
    use crate::encoding::{FileTransmitter, FileReceiver};
    use crate::net;

    // Connection handling
    pub fn connect(connection: &mut net::Connection, ip: IpAddr, port: u16) {
            *connection = net::Connection::new(ip, port);
    }

    pub fn disconnect(stream: &mut TcpStream, _transmitter: &mut FileTransmitter, _receiver: &mut FileReceiver) {
        let mut packet = [0; net::PACKET_SIZE];
        packet[0] = net::Code::Disconnect as u8;
        stream.write_all(&packet).expect("Network error");
    }

    // User commands
    pub fn upload(transmitter: &mut FileTransmitter, stream: &mut TcpStream, path: &Path) {
        let upload_packet = net::create_upload(path.file_name().unwrap().to_str().unwrap(), 0x1);
        stream.write_all(&upload_packet).expect("Unable to write to stream");
        let stats = transmitter.host_file(path.to_str().unwrap(), stream);
        println!("{}", stats);
    }

    pub fn download(receiver: &mut FileReceiver, stream: &mut TcpStream, path: &str) {
        let download_packet = net::create_download(path);
        stream.write_all(&download_packet).expect("Unable to write to stream");
        receiver.listen(stream);
    }

    pub fn delete(receiver: &mut FileReceiver, stream: &mut TcpStream, path: &str) {
            let delete_packet = net::create_delete(path);
            stream.write_all(&delete_packet).expect("Network error");
            receiver.listen(stream);
    }

    pub fn dir(receiver: &mut FileReceiver, stream: &mut TcpStream) {
        let dir_packet = net::create_dir("");
        stream.write_all(&dir_packet).expect("Network error");
        receiver.listen(stream);
    }
}

mod shell {
    use std::path::Path;
    use std::error::Error;
    use std::net::TcpStream;
    use std::io::{self, Write};
    use crate::encoding::{FileTransmitter, FileReceiver};
    use crate::net::client::{error, commands};
    use crate::net;

    //Commands
    fn upload(transmitter: &mut FileTransmitter, args: Vec<&str>, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            let path = Path::new(args[0]);
            commands::upload(transmitter, stream, &path);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }

    fn download(receiver: &mut FileReceiver, args: Vec<&str>, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            commands::download(receiver, stream, args[0]);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }

    fn delete(args: Vec<&str>, receiver: &mut FileReceiver, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            commands::delete(receiver, stream, args[0]);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }

    fn dir(args: Vec<&str>, receiver: &mut FileReceiver, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        if args.is_empty() {
            commands::dir(receiver, stream);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 0 arguments")))
        }
    }

    // Connection handling
    fn connect(args: Vec<&str>, connection: &mut net::Connection) -> Result<(), Box<dyn Error>> {
        if args.len() == 2 {
            let ip: std::net::IpAddr = args[0].parse().unwrap();
            let port: u16 = args[1].parse().unwrap();

            commands::connect(connection, ip, port);
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 2 arguments")))
        }
    }

    // Command parsing and running
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
            "delete" => { delete(args, receiver, stream) },
            "dir" => { dir(args, receiver, stream) }
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

    // Shell loop
    fn client_prompt(name: &str) {
        print!("({}) > ", name);
        io::stdout().flush().unwrap();
    }

    pub fn pre_connection_shell() -> net::Connection {
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
    pub fn post_connection_shell(stream: TcpStream, transmitter: FileTransmitter, receiver: FileReceiver) {
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
        commands::disconnect(&mut stream, &mut transmitter, &mut receiver);
    }
}

use crate::net;
use std::path::Path;
use crate::encoding::{FileTransmitter, FileReceiver};

// Connection handling
fn open_connection(ip_str: &str, port: u16) -> net::Connection {
    let ip: std::net::IpAddr = ip_str.parse().unwrap();
    net::Connection::new(ip, port)
}

// Start the client
pub fn start_client(matches: &clap::ArgMatches) {
    let connection = if matches.is_present("host") && matches.is_present("port") {
        let host = matches.value_of("host").unwrap();
        let port: u16 = matches.value_of("port").unwrap().parse().expect("Please provide a valid port");
        open_connection(host, port)
    }
    else {
        shell::pre_connection_shell()
    };
    let mut transmitter = FileTransmitter::new();
    let mut receiver = FileReceiver::new();
    let mut stream = connection.stream.expect("This should never happen");

    let mut had_cmd = false;

    if matches.is_present("list") {
        commands::dir(&mut receiver, &mut stream); 
        had_cmd = true;
    }

    if matches.is_present("download") {
        let path = matches.value_of("download").unwrap();
        commands::download(&mut receiver, &mut stream, path); 
        had_cmd = true;
    }

    if matches.is_present("upload") {
        let path = Path::new(matches.value_of("upload").unwrap());
        commands::upload(&mut transmitter, &mut stream, &path); 
        had_cmd = true;
    }

    if matches.is_present("delete") {
        let path = matches.value_of("delete").unwrap();
        commands::delete(&mut receiver, &mut stream, path); 
        had_cmd = true;
    }

    if matches.is_present("shell") || !had_cmd {
        shell::post_connection_shell(stream, transmitter, receiver);
    }
    else {
        commands::disconnect(&mut stream, &mut transmitter, &mut receiver);
    }
}
