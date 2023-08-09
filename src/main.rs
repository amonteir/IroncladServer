use std::env;
use std::error::Error;
use std::process;
pub mod cli;
pub mod error;
use ironcladserver::cli::{Config, HelpMenu, ServerCommand};
use ironcladserver::{Server, ServerConcurrency};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli_input: Vec<String> = env::args().collect();
    let config = Config::build(&cli_input).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(0);
    });

    match config.command {
        ServerCommand::Start => {
            let server = Server::init(config)?;
            match server.concurrency {
                //ServerConcurrency::RunningAsync => server.start_async().await?,
                ServerConcurrency::RunningAsync => server.start_async_tls().await?,
                ServerConcurrency::RunningThreadPool => server.start_tp()?,
            }
        }
        ServerCommand::Help => {
            HelpMenu::show();
        }
    }

    //println!("Shutting down.");
    Ok(())
}
