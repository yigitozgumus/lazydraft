use command::{parse_command, Command};
use config::{validate_config, Config};
use std::env;
use writing::{create_writing_list, print_writing_list};

mod command;
mod config;
mod writing;

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
    match create_writing_list(config) {
        Ok(writings) => print_writing_list(writings),
        Err(_) => exit_with_message("Couldn't print the writing list"),
    }
    Ok(())
}

fn execute_stage_command() {
    exit_with_message("stage command is called");
}

fn execute_config_command() {
    exit_with_message("config command is called");
}
