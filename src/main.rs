use std::env;
use std::error::Error;
use std::process;

pub mod cli;
pub mod error;
use boowebserver::cli::{Config, HelpMenu, ServerCommand};
use boowebserver::Server;

fn main() -> Result<(), Box<dyn Error>> {
    let cli_input: Vec<String> = env::args().collect();
    let config = Config::build(&cli_input).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(0);
    });

    match config.command {
        ServerCommand::Start => {
            let server = Server::init(config)?;
            server.start1().start()?;
        }
        ServerCommand::Help => {
            HelpMenu::show();
        }
    }

    //println!("Shutting down.");
    Ok(())
}
