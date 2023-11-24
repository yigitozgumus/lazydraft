use command::{parse_command, Command};
use config::{validate_config, Config};
use std::{
    env,
    fs::{self},
};

mod command;
mod config;

fn main() {
    match validate_config() {
        Ok(config) => {
            if config.has_empty_fields() {
                exit_with_message("There is an empty field in config. Please update it")
            } else {
                let args: Vec<String> = env::args().collect();
                if args.len() == 2 {
                    let argument = &args[1];
                    match parse_command(argument) {
                        Some(command) => match command {
                            Command::List => {
                                execute_list_command(&config).unwrap();
                            }
                            Command::Stage => {
                                execute_stage_command();
                            }
                            Command::Config => {
                                execute_config_command();
                            }
                        },
                        None => exit_with_message("Invalid Command"),
                    }
                } else {
                    exit_with_message("Invalid argument passing")
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn exit_with_message(message: &str) {
    println!("{}", message);
    std::process::exit(1);
}

fn execute_list_command(config: &Config) -> std::io::Result<()> {
    let entries = fs::read_dir(&config.source_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Invalid file name"))?;
        let file_name_str = file_name.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Invalid UTF-8 in file name")
        })?;
        println!("{}", file_name_str);
    }
    Ok(())
}

fn execute_stage_command() {
    exit_with_message("stage command is called");
}

fn execute_config_command() {
    exit_with_message("config command is called");
}
