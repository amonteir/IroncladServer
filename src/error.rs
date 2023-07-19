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
