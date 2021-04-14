use clap::{App};
use std::io::{self, Write};

fn start_server() {

}

mod client {
    use std::io::{self, Write};
    use std::error::Error;
    use colour;

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

    struct Connection {
        name: String
    }

    impl Connection {
        fn new() -> Connection {
            Connection{name: String::from("no connection")}
        }
    }

    fn client_prompt(connection: &Connection) {
        print!("({}) > ", connection.name);
        io::stdout().flush().unwrap();
    }

    fn connect(args: Vec<&str>) -> Result<(), Box<dyn Error>> {
        if args.len() == 2 {
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

    fn run_command(command: &str, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
        match command {
            "connect" => { connect(args) },
            "upload" => { upload(args) },
            "download" => { download(args) },
            "delete" => { delete(args) },
            "dir" => { dir(args) }
            _ => { println!("Invalid command"); Ok(()) }
        }
    }

    fn parse_command(command: &String) -> (String, Vec<&str>) {
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
        let mut connection = Connection::new();

        loop {
            client_prompt(&connection);
            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");

            let (command, args) = parse_command(&line);
            match run_command(&command, args) {
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
        start_server();
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
