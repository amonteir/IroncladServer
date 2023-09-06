use std::env;
use std::error::Error;
use std::process;
pub mod cli;
pub mod error;
use ironcladserver::cli::{Config, HelpMenu, ServerCommand, Version};
use ironcladserver::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli_input: Vec<String> = env::args().collect();
    let config = Config::build(&cli_input).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(0);
    });

    match config.command {
        ServerCommand::Start => {
            let server = Server::init(config.args_opts_map.unwrap())?;
            match server.with_tls {
                true => server.start_async_tls().await?,
                false => server.start_async().await?,
            }
        }
        ServerCommand::Help => {
            HelpMenu::show();
        }
        ServerCommand::Version => {
            Version::show();
        }
    }

    //println!("Shutting down.");
    Ok(())
}
