use clap::{App};
use std::io::{self, Write};

fn start_server() {

}

mod client {
    use std::io::{self, Write};
    fn client_prompt() {
        print!("> ");
        io::stdout().flush().unwrap();
    }

    fn run_command(command: &str, args: Vec<&str>) {
        println!("{} is command", command);
        match command {
            "connect" => {},
            "upload" => {},
            "download" => {},
            "delete" => {},

            "dir" => {}
            _ => { println!("Invalid command"); }
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
        loop {
            client_prompt();
            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");

            let (command, args) = parse_command(&line);

            run_command(&command, args);
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
