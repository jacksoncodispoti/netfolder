use std::net::{TcpListener, TcpStream, IpAddr, SocketAddr, Ipv4Addr};
use std::io::{Read, Write};
use crate::encoding::{FileReceiver, FileTransmitter};
use crate::net::{self, Code, parse, create};
use std::thread;

// Listen for connections and create new thread on connection start
pub struct ConnectionListener {
    _name: String,
    listener: TcpListener
}

impl ConnectionListener {
    pub fn new(name: &str, ip: IpAddr, port: u16) -> ConnectionListener {
        let addr = SocketAddr::from((ip, port));
        ConnectionListener{ _name: String::from(name), listener: TcpListener::bind(addr).unwrap() }
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

// Server connection
struct Connection {
    stream: TcpStream,
}

impl Connection {
    fn new(stream: TcpStream) -> Connection {
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
                        let code = parse::packet(&buf);

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
                let (name, id) = parse::upload(&packet);
                println!("[{}] Receiving upload: {}", addr, name);
                let stats = receiver.get_file(&name, id, &mut self.stream);
                println!("[{}]\t{}: {}", addr, name, stats);
                net::Code::Okay.packet()
            },
            Code::Delete => {
                let arg = parse::delete(packet);
                let _res = receiver.delete_file(&mut self.stream, &arg);
                net::Code::Okay.packet()
            },
            Code::Dir => {
                let _arg = parse::dir(packet);
                let _res = transmitter.dir("./", &mut self.stream);
                net::Code::Okay.packet()
            },
            Code::Redirect => {
                let (port, filename) = parse::redirect(packet);
                let stats = receiver.get_file(&filename, port, &mut self.stream);
                println!("[{}] {}", addr, stats);
                net::Code::Okay.packet()
            },
            Code::Download => {
                let path = parse::download(&packet);
                self.stream.write_all(&create::redirect(&path, 0)).expect("Network error");
                let _stats = transmitter.host_file(&path, &mut self.stream);
                net::Code::Okay.packet()
            },
            _ => { println!("[{}] Unknown command!", addr); net::Code::Error.packet() }
        }
    }
}

// Start server
pub fn start_server(_matches: &clap::ArgMatches) {
    let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let port = 3219;
    let listener = ConnectionListener::new("TheBlackPearl", ip, port);

    listener.connection_loop();
}
