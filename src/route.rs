#![forbid(unsafe_code)]
use crate::status; // Response status codes
use async_std::task;
use once_cell::sync::Lazy;
use std::path::Path;
use std::{thread, time::Duration};

pub enum Route {
    Homepage,
    Login,
    BadRequest,
}

// HTML files
pub static PATH_TO_HOME: Lazy<&Path> = Lazy::new(|| Path::new("resources/html/home.html"));
pub static PATH_TO_404: Lazy<&Path> = Lazy::new(|| Path::new("resources/html/404.html"));
pub static PATH_TO_LOGIN: Lazy<&Path> = Lazy::new(|| Path::new("resources/html/login.html"));
// Requests
static REQUEST_GET_HOME: &[u8; 16] = b"GET / HTTP/1.1\r\n";
static REQUEST_GET_SLEEP: &[u8; 21] = b"GET /sleep HTTP/1.1\r\n";
static REQUEST_POST_LOGIN: &[u8] = b"POST /login";

pub async fn route_request_async(buffer: &[u8]) -> (&'static str, &'static Path) {
    let (status_line, filename) = if buffer.starts_with(REQUEST_GET_HOME) {
        (status::STATUS_200, *PATH_TO_HOME)
    } else if buffer.starts_with(REQUEST_GET_SLEEP) {
        task::sleep(Duration::from_secs(5)).await;
        (status::STATUS_200, *PATH_TO_HOME)
    } else if buffer.starts_with(REQUEST_POST_LOGIN) {
        (status::STATUS_200, *PATH_TO_LOGIN)
    } else {
        (status::STATUS_404, *PATH_TO_404)
    };
    (status_line, filename)
}

pub async fn route_request_async1(buffer: &[u8]) -> (&'static str, Route) {
    if let Ok(request_str) = std::str::from_utf8(buffer) {
        println!("Received request:\n{}", request_str);
    } else {
        println!("Received non-UTF8 request data.");
    }

    if buffer.starts_with(REQUEST_GET_HOME) {
        (status::STATUS_200, Route::Homepage)
    } else if buffer.starts_with(REQUEST_GET_SLEEP) {
        task::sleep(Duration::from_secs(5)).await;
        (status::STATUS_200, Route::Homepage)
    } else if buffer.starts_with(REQUEST_POST_LOGIN) {
        (status::STATUS_200, Route::Login)
    } else {
        (status::STATUS_404, Route::BadRequest)
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
