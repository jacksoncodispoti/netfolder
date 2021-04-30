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
