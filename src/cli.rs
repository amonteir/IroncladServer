use crate::error::ConfigError;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub program: &'static str,
    pub command: ServerCommand,
    pub args_opts_map: Option<HashMap<ServerConfigArguments, String>>,
}

#[derive(Debug, PartialEq)]
pub enum ServerCommand {
    Help,
    Start,
    Version,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ServerConfigArguments {
    IpAddress,
    Port,
    Tls,
    Verbose,
}

pub struct HelpMenu {}

impl HelpMenu {
    pub fn show() {
        let help_menu = r#"
        Usage: ironcladserver COMMAND [FLAGS] [OPTIONS]
    
        Commands:
          help              Show this help message and exit
          start             Start the web server
          version           Show program's version number and exit
          
        Options ('*' means mandatory):
          -ip               * Input IP address of the web server, e.g. '-ip 127.0.0.1'
          -p                * Input listening port of the web server, e.g. '-p 8080'

        Flags:
          --notls           Does not run TLS.
          --v, --verbose    Outputs a lot more info to the console!  
    
        Usage example:
          ironcladserver start -ip 127.0.0.1 -p 7878
          ironcladserver start -ip 127.0.0.1 -p 7878 --insecure
          ironcladserver help
          ironcladserver version
        "#;

        println!("{}", help_menu);
    }
}

pub struct Version {}

impl Version {
    pub fn show() {
        println!("Version: 0.2.0.");
    }
}

impl Config {
    pub fn build(cli_input: &[String]) -> Result<Config, ConfigError> {
        if cli_input.len() <= 1 {
            return Err(ConfigError::NotEnoughArguments);
        }

        let mut args_opts_map: HashMap<ServerConfigArguments, String> = HashMap::new();
        let cli_program_name: &str = "ironcladserver";

        let cli_command = match cli_input[1].to_lowercase().as_str() {
            "help" => ServerCommand::Help,
            "start" => ServerCommand::Start,
            "version" => ServerCommand::Version,
            _ => return Err(ConfigError::UnknownCommand(cli_input[1].to_string())),
        };

        if cli_command == ServerCommand::Help || cli_command == ServerCommand::Version {
            return Ok(Config {
                program: cli_program_name,
                command: cli_command,
                args_opts_map: None,
            });
        }

        Config::parse_args_opts(cli_input, &mut args_opts_map)?;

        if !args_opts_map.contains_key(&ServerConfigArguments::IpAddress) {
            return Err(ConfigError::MissingOption("-ip".to_string()));
        }
        if !args_opts_map.contains_key(&ServerConfigArguments::Port) {
            return Err(ConfigError::MissingOption("-p".to_string()));
        }

        Ok(Config {
            program: cli_program_name,
            command: cli_command,
            args_opts_map: Some(args_opts_map),
        })
    }

    fn parse_args_opts(
        cli_input: &[String],
        args_opts_map: &mut HashMap<ServerConfigArguments, String>,
    ) -> Result<(), ConfigError> {
        let mut index: usize = 2;

        while index < cli_input.len() {
            match cli_input[index].to_lowercase().as_str() {
                "-ip" => {
                    // server ip
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        args_opts_map.entry(ServerConfigArguments::IpAddress)
                    {
                        e.insert(cli_input[index + 1].clone());
                        index += 1;
                    } else {
                        return Err(ConfigError::ParseError(
                            "ip address option '-ip' is allowed once".to_string(),
                        ));
                    }
                }
                "-p" => {
                    // server port
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        args_opts_map.entry(ServerConfigArguments::Port)
                    {
                        e.insert(cli_input[index + 1].clone());
                        index += 1;
                    } else {
                        return Err(ConfigError::ParseError(
                            "port option '-p' is allowed once".to_string(),
                        ));
                    }
                }
                "--notls" => {
                    // tls bool
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        args_opts_map.entry(ServerConfigArguments::Tls)
                    {
                        e.insert("false".to_string());
                    } else {
                        return Err(ConfigError::ParseError(
                            "flag '-notls' is allowed once".to_string(),
                        ));
                    }
                }
                "--v" | "--verbose" => {
                    // verbose
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        args_opts_map.entry(ServerConfigArguments::Verbose)
                    {
                        e.insert("true".to_string());
                    } else {
                        return Err(ConfigError::ParseError(
                            "flag '--v' or '--verbose' is allowed once".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(ConfigError::UnknownCommand(
                        "option or flag not available.".to_string(),
                    ))
                }
            }
            index += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Config, ServerConfigArguments};

    #[test]
    fn check_cli_input() {
        let mut config_args_opts_map: HashMap<ServerConfigArguments, String> = HashMap::new();
        let cli_input = vec![
            "ironcladserver".to_string(),
            "start".to_string(),
            "-ip".to_string(),
            "192.168.0.1".to_string(),
            "-p".to_string(),
            "7878".to_string(),
        ];

        match Config::parse_args_opts(&cli_input, &mut config_args_opts_map) {
            Ok(_) => {
                if let Some(ip) = config_args_opts_map.get(&ServerConfigArguments::IpAddress) {
                    println!("IP: {}", ip);
                    assert_eq!(*ip, "192.168.0.1".to_string());
                } else {
                    panic!("Fix 'cli_input' vector.")
                }
                if let Some(ip) = config_args_opts_map.get(&ServerConfigArguments::Port) {
                    println!("IP: {}", ip);
                    assert_eq!(*ip, "7878".to_string());
                } else {
                    panic!("Fix 'cli_input' vector.")
                }
            }
            Err(e) => panic!("Error: {}.", e),
        }
    }
}
