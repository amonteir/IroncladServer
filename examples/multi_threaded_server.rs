/// This example creates a web server with a threads pool.  
//  As an example, to run this use:
//    "cargo run --example multi_threaded_server start -ip 127.0.0.1 -p 7878 -tp 10"
extern crate boowebserver;
use boowebserver::cli::{Config, HelpMenu, ServerCommand};
use boowebserver::Server;
use std::env;
use std::error::Error;
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    let cli_input: Vec<String> = env::args().collect();
    let config = Config::build(&cli_input).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(0);
    });

    match config.command {
        ServerCommand::Start => {
            let server = Server::init(config)?;
            server.start_tp()?;
        }
        ServerCommand::Help => {
            HelpMenu::show();
        }
    }

    //println!("Shutting down.");
    Ok(())
}
