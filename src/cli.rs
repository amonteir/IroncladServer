use crate::error::ConfigError;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub program: &'static str,
    pub command: ServerCommand,
    pub args_opts_map: HashMap<ServerConfigArguments, String>,
}

#[derive(Debug, PartialEq)]
pub enum ServerCommand {
    Help,
    Start,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ServerConfigArguments {
    IpAddress,
    Port,
    ThreadPool,
}

pub struct HelpMenu {}

impl HelpMenu {
    pub fn show() {
        let help_menu = r#"
        Usage: boowebserver COMMAND [OPTIONS]
    
        Commands:
          help          Show this help message and exit
          start         Start the web server
          
        Options:
          -ip           Input IP address of the web server, e.g. '-ip 127.0.0.1'
          -p            Input listening port of the web server, e.g. '-p 8080' 
          -tp           Input thread pool size, e.g. '-tp 10'
          -v            Show program's version number and exit
    
        Usage example:
          boowebserver start -ip 127.0.0.1 -p 8080 -tp 10
        "#;

        println!("{}", help_menu);
    }
}

impl Config {
    pub fn build(cli_input: &[String]) -> Result<Config, ConfigError> {
        if cli_input.len() <= 1 {
            return Err(ConfigError::NotEnoughArguments);
        }

        let mut args_opts_map: HashMap<ServerConfigArguments, String> = HashMap::new();
        let cli_program_name: &str = "boowebserver";

        let cli_command = match cli_input[1].to_lowercase().as_str() {
            "help" => ServerCommand::Help,
            "start" => ServerCommand::Start,
            _ => return Err(ConfigError::UnknownCommand(cli_input[1].to_string())),
        };

        if cli_command == ServerCommand::Help {
            return Ok(Config {
                program: cli_program_name,
                command: cli_command,
                args_opts_map,
            });
        }

        Config::parse_args_opts(cli_input, &mut args_opts_map)?;
        //.map_err(|err| ConfigError::ParseError(err.to_string()))?;

        if !args_opts_map.contains_key(&ServerConfigArguments::IpAddress) {
            return Err(ConfigError::MissingOption("-ip".to_string()));
        }
        if !args_opts_map.contains_key(&ServerConfigArguments::Port) {
            return Err(ConfigError::MissingOption("-p".to_string()));
        }

        Ok(Config {
            program: cli_program_name,
            command: cli_command,
            args_opts_map,
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
                    if let std::collections::hash_map::Entry::Vacant(e) = args_opts_map.entry(ServerConfigArguments::IpAddress) {
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
                    if let std::collections::hash_map::Entry::Vacant(e) = args_opts_map.entry(ServerConfigArguments::Port) {
                        e.insert(cli_input[index + 1].clone());
                        index += 1;
                    } else {
                        return Err(ConfigError::ParseError(
                            "port option '-p' is allowed once".to_string(),
                        ));
                    }
                }
                "-tp" => {
                    // concurrency: thread pool
                    if let std::collections::hash_map::Entry::Vacant(e) = args_opts_map.entry(ServerConfigArguments::ThreadPool) {        
                        e.insert(cli_input[index + 1].clone());
                        index += 1;
                    } else {
                        return Err(ConfigError::ParseError(
                            "threads pool option '-tp' is allowed once".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(ConfigError::UnknownCommand(
                        "option not available.".to_string(),
                    ))
                }
            }
            index += 1;
        }
        Ok(())
    }
}
