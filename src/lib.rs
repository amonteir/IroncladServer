#![forbid(unsafe_code)]

use std::{error::Error, fs, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::rustls;
use tokio_rustls::TlsAcceptor;

pub mod cli;
pub mod error;
pub mod models;
pub mod psql;
pub mod route;
pub mod status;
use crate::cli::ServerConfigArguments;
use crate::route::{handle_connection_async, TcpStreamType};
use std::collections::HashMap;

pub struct Server {
    ip_port: String,
    pub with_tls: bool,
    pub verbose: bool,
}

impl Server {
    /// Reads a ip address, port and concurrency settings from Config (i.e. user cli input)
    /// and returns the Server object
    ///
    pub fn init(
        opts_flags: HashMap<ServerConfigArguments, String>,
    ) -> Result<Server, Box<dyn Error>> {
        let ip_addr = opts_flags.get(&ServerConfigArguments::IpAddress).unwrap();
        let port = opts_flags.get(&ServerConfigArguments::Port).unwrap();
        let ip_port = format!("{}:{}", ip_addr, port);
        let with_tls = opts_flags.get(&ServerConfigArguments::Tls).is_none();
        let verbose = opts_flags.get(&ServerConfigArguments::Verbose).is_some();

        Ok(Server {
            ip_port,
            with_tls,
            verbose,
        })
    }

    /// Starts the server using async
    ///
    pub async fn start_async(&self) -> Result<(), Box<dyn Error>> {
        let listener = tokio::net::TcpListener::bind(&self.ip_port).await?;
        println!("[  OK  ]     Started the server and serving requests using async, no TLS.");

        loop {
            let (socket, _) = listener.accept().await?;

            tokio::spawn(async move {
                // Process each socket concurrently.
                handle_connection_async(&mut TcpStreamType::TokioNoTls(socket)).await;
            });
        }
    }
    /// Starts the server using async and tls
    ///
    pub async fn start_async_tls(&self) -> Result<(), Box<dyn Error>> {
        let certs = load_certs("certs/sample.pem")?;
        let key = load_private_key("certs/sample.rsa")?;

        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

        let acceptor = TlsAcceptor::from(Arc::new(config));
        let listener = TcpListener::bind(&self.ip_port).await?;
        println!("[  OK  ]     Started the TLS server in async mode.");

        loop {
            match listener.accept().await {
                Ok((socket, _ip_addr)) => {
                    let acceptor = acceptor.clone();
                    tokio::spawn(async move {
                        match acceptor.accept(socket).await {
                            Ok(tls_stream) => {
                                handle_connection_async(&mut TcpStreamType::TokioTls(tls_stream))
                                    .await;
                            }
                            Err(e) => {
                                println!("TLS handshake error: {}", e);
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Accept failed = {:?}", e),
            }
        }
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::collections::HashMap;

//     #[tokio::test]
//     async fn invalid_server_ip() {
//         let mut config_args_opts_map: HashMap<ServerConfigArguments, String> = HashMap::new();
//         config_args_opts_map.insert(ServerConfigArguments::IpAddress, String::from("127.0.1"));
//         config_args_opts_map.insert(ServerConfigArguments::Port, String::from("7878"));

//         let server = Server::init(config_args_opts_map).unwrap();
//         let result = server.start_async().await;

//         if let Err(e) = result {
//             assert_eq!(e.to_string(), "No such host is known. (os error 11001)");
//         } else {
//             panic!("Expected Err, but got Ok");
//         }
//     }
// }
