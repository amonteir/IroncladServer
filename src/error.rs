use sqlx::Error as sqlxerror;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    NotEnoughArguments,
    UnknownCommand(String),
    MissingOption(String),
    ParseError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let suffix = String::from("\nFor syntax help, type 'boowebserver help'.");

        match self {
            ConfigError::NotEnoughArguments => write!(f, "Not enough arguments.{}", suffix),
            ConfigError::UnknownCommand(com) => write!(f, "Unknown command: {}{}", com, suffix),
            ConfigError::MissingOption(opt) => write!(f, "Missing option: {}{}", opt, suffix),
            ConfigError::ParseError(err) => write!(f, "Parse error: {}{}", err, suffix),
        }
    }
}

impl Error for ConfigError {} // ConfigError is of type Error

#[derive(Debug)]
pub enum PsqlError {
    SqlxError(sqlxerror),
    PasswordMismatch,
}

impl From<sqlxerror> for PsqlError {
    fn from(err: sqlxerror) -> Self {
        PsqlError::SqlxError(err)
    }
}

impl std::fmt::Display for PsqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PsqlError::SqlxError(err) => write!(f, "SQLx error: {}", err),
            PsqlError::PasswordMismatch => write!(f, "Passwords don't match"),
        }
    }
}

impl Error for PsqlError {}
