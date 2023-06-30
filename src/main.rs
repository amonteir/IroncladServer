use std::net::{TcpListener};
use boowebserver::{ThreadPool, handle_connection};
//use hello::Server;

fn main() {

    let listener = TcpListener::bind("127.0.1:7878");
    //let listener = Server::new("127.0.0.1:7878",10);
    let pool = ThreadPool::new(10);

    match (listener, pool) {
        (Ok(listener), Ok(pool)) => {
            // To test workers shutdown use: for stream in listener.incoming().take(2) {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    pool.execute(|| {
                        handle_connection(stream);
                    });
                } else {
                    eprintln!("Failed to read from incoming stream");

                }
            }
        }
        (Err(e), _) => eprintln!("Failed to bind TCP listener: {}", e),
        (_, Err(e)) => eprintln!("Failed to create thread pool: {}", e),
    }
    println!("Shutting down. AA");
}