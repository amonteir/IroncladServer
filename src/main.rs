use boowebserver::Server;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    let server = Server::new("127.0.0.1:7878",10)?;
    server.start()?;
    
    println!("Shutting down.");
    Ok(())
    
}