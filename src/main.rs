use clap::{App, Arg};
//use net::{server, client};
pub mod net;
pub mod stats;
pub mod encoding;
//

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
        net::server::start_server(server_matches);
        //println!("Running the server");
    }
    else if let Some(client_matches) = matches.subcommand_matches("client") {
        net::client::start_client(client_matches);
        //println!("Running the client");
    }
    else {
        println!("Please specify server or client");
    }
}
