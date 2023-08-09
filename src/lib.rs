use async_std::{prelude::*, task};
use async_tls::TlsAcceptor;
use futures::stream::StreamExt;
use rustls::{NoClientAuth, ServerConfig};
use std::{
    error::Error,
    fmt, fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    process,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

pub mod cli;
pub mod error;
use crate::cli::{Config, ServerConfigArguments};

pub enum ServerConcurrency {
    RunningAsync,
    RunningThreadPool,
}
pub struct Server {
    ip_port: String,
    pub concurrency: ServerConcurrency,
    workers_pool: Option<ThreadPool>,
}

impl Server {
    /// Reads a ip address, port and concurrency settings from Config (i.e. user cli input)
    /// and returns the Server object
    ///
    pub fn init(config: Config) -> Result<Server, Box<dyn Error>> {
        let ip_addr = config
            .args_opts_map
            .get(&ServerConfigArguments::IpAddress)
            .unwrap();
        let port = config
            .args_opts_map
            .get(&ServerConfigArguments::Port)
            .unwrap();
        let ip_port = format!("{}:{}", ip_addr, port);

        let (concurrency, workers_pool) =
            match config.args_opts_map.get(&ServerConfigArguments::ThreadPool) {
                Some(value) => {
                    let pool_size: usize = match value.parse() {
                        Ok(size) => size,
                        Err(_) => process::exit(0), // TODO: change this to an Error in error.rs
                    };
                    (
                        ServerConcurrency::RunningThreadPool,
                        Some(ThreadPool::new(pool_size)?),
                    )
                }
                None => (ServerConcurrency::RunningAsync, None),
            };
        Ok(Server {
            ip_port,
            concurrency,
            workers_pool,
        })
    }

    /// Starts the server with a thread pool
    pub fn start_tp(&self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.ip_port)?;
        println!("Started the server with a thread pool.");

        for stream in listener.incoming() {
            let stream = stream?;
            match &self.workers_pool {
                Some(pool) => {
                    pool.execute(|| {
                        handle_connection_tp(stream);
                    });
                }
                None => {
                    process::exit(0); // TODO: change this to an Error in error.rs
                }
            }
        }
        Ok(())
    }
    /// Starts the server using async
    ///
    pub async fn start_async(&self) -> Result<(), Box<dyn Error>> {
        let listener = async_std::net::TcpListener::bind(&self.ip_port).await?;
        println!("Started the server and serving requests using async.");

        listener
            .incoming()
            .for_each_concurrent(/* limit */ None, |tcpstream| async move {
                let tcpstream = tcpstream.unwrap();
                handle_connection_async(tcpstream).await;
            })
            .await;

        Ok(())
    }
    /// Starts the server using async and tls
    ///
    pub async fn start_async_tls(&self) -> Result<(), Box<dyn Error>> {
        let certs = load_certs("certs/sample.pem")?;
        let key = load_private_key("certs/sample.rsa")?;

        let mut config = ServerConfig::new(NoClientAuth::new());
        config.set_single_cert(certs, key).unwrap();

        let acceptor = TlsAcceptor::from(Arc::new(config));
        let listener = async_std::net::TcpListener::bind(&self.ip_port).await?;
        println!("Started the tls server in async mode.");
        listener
            .incoming()
            .for_each_concurrent(/* limit */ None, {
                let acceptor = acceptor.clone();
                move |tcpstream_result| {
                    let acceptor = acceptor.clone();
                    async move {
                        if let Ok(tcpstream) = tcpstream_result {
                            match acceptor.accept(tcpstream).await {
                                Ok(stream) => handle_connection_async_tls(stream).await,
                                Err(e) => {
                                    println!("TLS handshake error: {}", e);
                                }
                            }
                        } else {
                            println!("Error accepting tcp stream.");
                        }
                    }
                }
            })
            .await;
        Ok(())
    }
    /// Chaining POC
    pub fn start1(self) -> Self {
        println!("ttttest");
        self
    }
}

fn error(err: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err)
}

// Load public certificate from file.
fn load_certs(filename: &str) -> std::io::Result<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = std::io::BufReader::new(certfile);

    // Load and return certificate.
    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|_| error("failed to load certificate".into()))?;
    Ok(certs.into_iter().map(rustls::Certificate).collect())
}

// Load private key from file.
fn load_private_key(filename: &str) -> std::io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = std::io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = rustls_pemfile::rsa_private_keys(&mut reader)
        .map_err(|_| error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(error("expected a single private key".into()));
    }
    Ok(rustls::PrivateKey(keys[0].clone()))
}

async fn handle_connection_async_tls(
    mut stream: async_tls::server::TlsStream<async_std::net::TcpStream>,
) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer).await {
        Ok(bytes_read) => {
            println!("Read {} bytes", bytes_read);

            let get = b"GET / HTTP/1.1\r\n";
            let sleep = b"GET /sleep HTTP/1.1\r\n";

            let (status_line, filename) = if buffer.starts_with(get) {
                ("HTTP/1.1 200 OK\r\n\r\n", r"resources\html\home.html")
            } else if buffer.starts_with(sleep) {
                task::sleep(Duration::from_secs(5)).await;
                ("HTTP/1.1 200 OK\r\n\r\n", r"resources\html\home.html")
            } else {
                ("HTTP/1.1 404 NOT FOUND\r\n\r\n", r"resources\html\404.html")
            };

            if let Ok(contents) = fs::read_to_string(filename) {
                let response = format!("{}{}", status_line, contents);
                if let Err(e) = stream.write(response.as_bytes()).await {
                    eprintln!("Error writing to stream: {}", e);
                }
                if let Err(e) = stream.flush().await {
                    eprintln!("Error flushing stream: {}", e);
                }
            } else {
                eprintln!("Error reading file: {}", filename);
            }
        }
        Err(e) => {
            // Handle the error in some way.
            eprintln!("Error reading from stream: {}", e);
        }
    }
}

async fn handle_connection_async(mut stream: async_std::net::TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer).await {
        Ok(_bytes_read) => {
            //println!("Read {} bytes", bytes_read);

            let get = b"GET / HTTP/1.1\r\n";
            let sleep = b"GET /sleep HTTP/1.1\r\n";

            let (status_line, filename) = if buffer.starts_with(get) {
                ("HTTP/1.1 200 OK\r\n\r\n", r"resources\html\home.html")
            } else if buffer.starts_with(sleep) {
                task::sleep(Duration::from_secs(5)).await;
                ("HTTP/1.1 200 OK\r\n\r\n", r"resources\html\home.html")
            } else {
                ("HTTP/1.1 404 NOT FOUND\r\n\r\n", r"resources\html\404.html")
            };
            let contents = fs::read_to_string(filename).unwrap();

            let response = format!("{status_line}{contents}");
            stream.write(response.as_bytes()).await.unwrap();
            stream.flush().await.unwrap();
        }
        Err(e) => {
            // Handle the error in some way.
            eprintln!("Error reading from stream: {}", e);
        }
    }
}

fn handle_connection_tp(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_bytes_read) => {
            //println!("Read {} bytes", bytes_read);

            let get = b"GET / HTTP/1.1\r\n";
            let sleep = b"GET /sleep HTTP/1.1\r\n";

            let (status_line, filename) = if buffer.starts_with(get) {
                ("HTTP/1.1 200 OK", r"resources\html\home.html")
            } else if buffer.starts_with(sleep) {
                thread::sleep(Duration::from_secs(5));
                ("HTTP/1.1 200 OK", r"resources\html\home.html")
            } else {
                ("HTTP/1.1 404 NOT FOUND", r"resources\html\404.html")
            };

            let contents = fs::read_to_string(filename).unwrap();

            let response = format!(
                "{}\r\nContent-Length: {}\r\n\r\n{}",
                status_line,
                contents.len(),
                contents
            );

            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        Err(e) => {
            // Handle the error in some way.
            eprintln!("Error reading from stream: {}", e);
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
struct PoolCreationError {
    msg: String,
}

impl PoolCreationError {
    fn new(msg: &str) -> Self {
        PoolCreationError {
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for PoolCreationError {} // PoolCreationError is of type Error. No need to override existing Error methods

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size < 1 {
            return Err(PoolCreationError::new(
                "Number of workers in pool must be greater than 0.",
            ));
        }

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn invalid_server_ip() {
        let mut config_args_opts_map: HashMap<ServerConfigArguments, String> = HashMap::new();
        config_args_opts_map.insert(ServerConfigArguments::IpAddress, String::from("127.0.1"));
        config_args_opts_map.insert(ServerConfigArguments::Port, String::from("7878"));
        config_args_opts_map.insert(ServerConfigArguments::ThreadPool, String::from("10"));
        let test_config: Config = Config {
            program: "ironcladserver",
            command: cli::ServerCommand::Start,
            args_opts_map: config_args_opts_map,
        };

        let server = Server::init(test_config).unwrap();
        let result = server.start_tp();

        if let Err(e) = result {
            assert_eq!(e.to_string(), "No such host is known. (os error 11001)");
        } else {
            panic!("Expected Err, but got Ok");
        }
    }
}
