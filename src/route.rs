#![forbid(unsafe_code)]

use crate::error::PsqlError;
use crate::models::LoginPayload;
use crate::psql::db_psql_validate_user;
use crate::status; // Response status codes
use async_std::prelude::*;
use async_std::task;
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::postgres::PgPool;
use std::path::Path;
use std::{env, fs, thread, time::Duration};

pub enum Route {
    Homepage,
    Favicon,
    BadRequest,
    Login,
}

// HTML files
static PATH_TO_HOME: Lazy<&Path> = Lazy::new(|| Path::new("resources/html/home.html"));
static PATH_TO_404: Lazy<&Path> = Lazy::new(|| Path::new("resources/html/404.html"));
static PATH_TO_401: Lazy<&Path> = Lazy::new(|| Path::new("resources/html/401.html"));
static PATH_TO_LOGIN: Lazy<&Path> = Lazy::new(|| Path::new("resources/html/login.html"));
static PATH_TO_FAVICON: Lazy<&Path> = Lazy::new(|| Path::new("resources/html/favicon.ico"));
// Requests
static REQUEST_GET_HOME: &[u8; 16] = b"GET / HTTP/1.1\r\n";
static REQUEST_GET_SLEEP: &[u8; 21] = b"GET /sleep HTTP/1.1\r\n";
static REQUEST_POST_LOGIN: &[u8] = b"POST /login";
static REQUEST_GET_FAVICON: &[u8] = b"GET /favicon.ico HTTP/1.1\r\n";

pub async fn route_request_async(buffer: &[u8]) -> (&'static str, &'static Path) {
    let (status_line, filename) = if buffer.starts_with(REQUEST_GET_HOME) {
        (status::STATUS_200, *PATH_TO_HOME)
    } else if buffer.starts_with(REQUEST_GET_SLEEP) {
        task::sleep(Duration::from_secs(5)).await;
        (status::STATUS_200, *PATH_TO_HOME)
    } else if buffer.starts_with(REQUEST_POST_LOGIN) {
        (status::STATUS_200, *PATH_TO_LOGIN)
    } else if buffer.starts_with(REQUEST_GET_FAVICON) {
        (status::STATUS_200, *PATH_TO_FAVICON)
    } else {
        (status::STATUS_404, *PATH_TO_404)
    };
    (status_line, filename)
}

// If adding/removing headers, make sure the last header doesn't terminate in \r\n
// because that is already being added to the response string in fn 'build_http_response'
fn build_http_headers(security_enabled: bool, payload_length: usize, content_type: &str) -> String {
    let headers: String = match security_enabled {
        true => {
            format!(
                "Connection: keep-alive\r\n\
                Content-Type: {}\r\n\
                Access-Control-Allow-Origin: *\r\n\
                X-Content-Type-Options: nosniff\r\n\
                X-XSS-Protection: 1; mode=block\r\n\
                Content-Security-Policy: default-src 'self'\r\n\
                Content-Length: {}",
                content_type, payload_length
            )
        }
        false => {
            format!(
                "Connection: keep-alive\r\n\
                Content-Type: {}\r\n\
                Content-Length: {}",
                content_type, payload_length
            )
        }
    };
    headers
}

fn build_http_response<T: AsRef<[u8]>>(status: &str, payload: T, content_type: &str) -> String {
    let payload_bytes = payload.as_ref();
    let headers = build_http_headers(false, payload_bytes.len(), content_type);
    let response_payload = match payload_bytes {
        // If it's a UTF-8 string, format it as is.
        // This will work for String and &str types.
        bytes if std::str::from_utf8(bytes).is_ok() => {
            std::str::from_utf8(bytes).unwrap().to_string()
        }
        // If it's not a UTF-8 string (i.e., arbitrary binary data), format the bytes directly.
        // This will work for Vec<u8> and &[u8] types.
        bytes => format!("{:?}", bytes),
    };
    format!("{}\r\n{}\r\n\r\n{}", status, headers, response_payload)
}

fn build_http_response_login(status: &str) -> String {
    let payload = json!({
        "success": true
    });
    let payload_str = payload.to_string();
    let headers = build_http_headers(false, payload_str.as_bytes().len(), "application/json");
    let response = format!("{}\r\n{}\r\n\r\n{}", status, headers, payload_str);
    response
}

async fn process_request_async(
    stream: async_tls::server::TlsStream<async_std::net::TcpStream>,
    buffer: &[u8],
    bytes_read: usize,
) {
    let mut route: Route = if buffer.starts_with(REQUEST_GET_HOME) {
        Route::Homepage
    } else if buffer.starts_with(REQUEST_GET_SLEEP) {
        task::sleep(Duration::from_secs(5)).await;
        Route::Homepage
    } else if buffer.starts_with(REQUEST_POST_LOGIN) {
        Route::Login
    } else if buffer.starts_with(REQUEST_GET_FAVICON) {
        Route::Favicon
    } else {
        Route::BadRequest
    };
    //let http_request1 = String::from_utf8(buffer.to_vec());
    let http_request = String::from_utf8_lossy(&buffer[..bytes_read]);
    // http_request_split[0] = path + headers
    // http_request_split[1] = payload (if any, a GET for instance doesn't contain any payload)
    let http_request_split: Vec<&str> = http_request.split("\r\n\r\n").collect();

    if http_request_split.len() < 2 {
        eprintln!("Invalid HTTP request format");
        route = Route::BadRequest;
    }

    match route {
        Route::BadRequest => {
            if let Ok(contents) = fs::read_to_string(*PATH_TO_404) {
                let response =
                    build_http_response(status::STATUS_404, contents, "text/html; charset=UTF-8");
                write_to_client(stream, response).await;
            } else {
                eprintln!("Error reading file: 404.html");
            }
        }
        Route::Homepage => {
            if let Ok(contents) = fs::read_to_string(*PATH_TO_HOME) {
                let response =
                    build_http_response(status::STATUS_200, contents, "text/html; charset=UTF-8");
                write_to_client(stream, response).await;
            } else {
                eprintln!("Error reading file: home.html");
            }
        }
        Route::Favicon => {
            println!("favicon");
            if let Ok(contents) = fs::read(*PATH_TO_FAVICON) {
                let response = build_http_response(status::STATUS_200, contents, "image/x-icon");
                write_to_client(stream, response).await;
            } else {
                eprintln!("Error reading file: favicon.ico");
            }
        }
        Route::Login => {
            dotenv::dotenv().ok();
            let database_url =
                env::var("DATABASE_URL").expect("Failed to load 'DATABASE_URL' env variable");

            let http_payload = http_request_split[1];
            //println!("JSON sent by client: {}", http_payload);

            if let Ok(login_payload) = serde_json::from_str::<LoginPayload>(http_payload) {
                //println!("user:{}", &login_payload.username);
                if let Ok(user) = LoginPayload::new(login_payload.username, login_payload.pwd) {
                    if let Ok(pool) = PgPool::connect(database_url.as_str()).await {
                        match db_psql_validate_user(&pool, &user).await {
                            Ok(_) => {
                                write_to_client(
                                    stream,
                                    build_http_response_login(status::STATUS_200),
                                )
                                .await;
                            }
                            Err(PsqlError::SqlxError(sqlx::Error::RowNotFound)) => {
                                println!("User '{}' does not exist in database.", user.username);
                                if let Ok(contents) = fs::read_to_string(*PATH_TO_401) {
                                    write_to_client(
                                        stream,
                                        build_http_response(
                                            status::STATUS_401,
                                            contents,
                                            "text/html; charset=UTF-8",
                                        ),
                                    )
                                    .await;
                                } else {
                                    eprintln!("Error reading file: 401.html");
                                }
                            }
                            Err(PsqlError::PasswordMismatch) => {
                                println!(
                                    "{} for user '{}'.",
                                    PsqlError::PasswordMismatch,
                                    user.username
                                );
                                if let Ok(contents) = fs::read_to_string(*PATH_TO_401) {
                                    write_to_client(
                                        stream,
                                        build_http_response(
                                            status::STATUS_401,
                                            contents,
                                            "text/html; charset=UTF-8",
                                        ),
                                    )
                                    .await;
                                } else {
                                    eprintln!("Error reading file: 401.html");
                                }
                            }
                            Err(PsqlError::SqlxError(err)) => {
                                eprintln!("{}", err);
                                write_to_client(
                                    stream,
                                    build_http_response(
                                        status::STATUS_500,
                                        "",
                                        "text/html; charset=UTF-8",
                                    ),
                                )
                                .await;
                            }
                        }
                    }
                } else {
                    eprintln!("Failed to create a new user");
                }
            } else {
                eprintln!("Failed to parse JSON payload");
            }
        }
    }
}

async fn write_to_client(
    mut stream: async_tls::server::TlsStream<async_std::net::TcpStream>,
    response: String,
) {
    if let Err(e) = stream.write(response.as_bytes()).await {
        eprintln!("Error writing to stream: {}", e);
    }
    if let Err(e) = stream.flush().await {
        eprintln!("Error flushing stream: {}", e);
    }
}

pub fn route_request(buffer: &[u8]) -> (&str, &'static Path) {
    let (status_line, filename) = if buffer.starts_with(REQUEST_GET_HOME) {
        (status::STATUS_200, *PATH_TO_HOME)
    } else if buffer.starts_with(REQUEST_GET_SLEEP) {
        thread::sleep(Duration::from_secs(5));
        (status::STATUS_200, *PATH_TO_HOME)
    } else if buffer.starts_with(REQUEST_POST_LOGIN) {
        (status::STATUS_200, *PATH_TO_LOGIN)
    } else {
        (status::STATUS_404, *PATH_TO_404)
    };

    (status_line, filename)
}

pub async fn handle_connection_async_tls(
    mut stream: async_tls::server::TlsStream<async_std::net::TcpStream>,
) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer).await {
        Ok(bytes_read) => {
            //println!("Read {} bytes", bytes_read);
            process_request_async(stream, &buffer, bytes_read).await;
        }
        Err(e) => {
            eprintln!("Error reading from stream: {}", e);
        }
    }
}
