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

    pub const PACKET_SIZE: usize = 512;

    #[derive(Debug)]
    pub enum Code {
        Unknown=0x0,
        Upload=0x1,
        Download=0x2,
        Delete=0x3,
        Dir=0x4,
        Hello=0x5
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
                _ => Code::Unknown
            }
        }
    }

    pub fn parse_packet(packet: [u8; PACKET_SIZE]) -> Code {
        if packet.len() < 1 {
            Code::Unknown
        }
        else {
            Code::from_u8(packet[0])
        }
    }

    pub struct Connection {
        pub name: String,
        pub stream: Option<ConnectionStream>
    }
    impl Connection {
        pub fn new(ip: net::IpAddr, port: u16) -> Connection {
            Connection{name: String::from("Default name"), stream: Some(ConnectionStream::new(ip, port))}
        }
        pub fn default() -> Connection {
            Connection{ name: String::from("no connection"), stream: None }
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
   pub struct FileEncoder<'a> {
        stream: &'a mut TcpStream
   }

   impl FileEncoder<'_> {
       pub fn new<'a>(stream: &'a mut TcpStream) -> FileEncoder<'a> {
            FileEncoder {stream: stream}
       }
       pub fn add_upload(&self) {

       }

       pub fn add_receive(&self) {

       }
       
       pub fn add_send(&self) {

       }
   }
}
mod server {
    use std::net::{TcpListener, TcpStream, IpAddr, SocketAddr, Ipv4Addr};
    use crate::encoding::FileEncoder;
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
            let mut encoder = FileEncoder::new(&mut self.stream);
            let mut buf = [0; net::PACKET_SIZE ];
            loop {
                match self.stream.peek(&mut buf) {
                    Ok(size) => {
                        let code = net::parse_packet(buf);
                    },
                    Err(_e) => {}
                }
            }
        }

        fn handle_command(&mut self, encoder: FileEncoder, command: Code) {
            match command {
                Code::Upload => { encoder.add_receive(); },
                Code::Download => { encoder.add_send(); },
                Code::Delete => {},
                Code::Dir => {},
                Code::Hello => {},
                Code::Unknown => { println!("Unknown command!"); }
                _ => { println!("Unknown command!"); }
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
    use crate::net;
    use std::error::Error;
    use crate::error;
    use colour;


    fn client_prompt(connection: &net::Connection) {
        print!("({}) > ", connection.name);
        io::stdout().flush().unwrap();
    }

    fn connect(args: Vec<&str>, connection: &mut net::Connection) -> Result<(), Box<dyn Error>> {
        if args.len() == 2 {
            let ip: std::net::IpAddr = args[0].parse().unwrap();
            let port: u16 = args[1].parse().unwrap();

            *connection = net::Connection::new(ip, port);
            match &mut connection.stream {
                Some(stream) => {
                },
                None => {}
            };
            
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 2 arguments")))
        }
    }

    fn upload(args: Vec<&str>) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }
    fn download(args: Vec<&str>) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }
    fn delete(args: Vec<&str>) -> Result<(), Box<dyn Error>> {
        if args.len() == 1 {
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 1 argument")))
        }
    }
    fn dir(args: Vec<&str>) -> Result<(), Box<dyn Error>> {
        if args.len() == 0 {
            Ok(()) 
        }
        else {
            Err(Box::new(error::ArgError::new("Expected 0 arguments")))
        }
    }

    fn run_command(connection: &mut net::Connection, command: &str, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
        match command {
            "connect" => { connect(args, connection) },
            "upload" => { upload(args) },
            "download" => { download(args) },
            "delete" => { delete(args) },
            "dir" => { dir(args) }
            _ => { println!("Invalid command"); Ok(()) }
        }
    }

    fn parse_command<'a>(command: &'a String, connection: &mut net::Connection) -> (String, Vec<&'a str>) {
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
        let mut connection = net::Connection::default();

        loop {
            client_prompt(&connection);
            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");

            let (command, args) = parse_command(&line, &mut connection);
            match run_command(&mut connection, &command, args) {
                Ok(()) => {},
                Err(e)  => { colour::red_ln!("{:?}", e)}
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
