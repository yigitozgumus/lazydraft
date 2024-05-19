use asset::{get_asset_list_of_writing, transfer_asset_files};
use command::{parse_command, Command};
use config::{validate_config, Config};
use std::env;
use writing::{
    create_writing_list, print_writing_list, select_draft_writing_from_list,
    update_writing_content_and_transfer,
};

mod asset;
mod command;
mod config;
mod writing;

fn main() {
    match validate_config() {
        Ok(config) => {
            if config.has_empty_fields() {
                exit_with_message("There is an empty field in config. Please update it")
            }
            let args: Vec<String> = env::args().collect();
            if args.len() != 2 {
                exit_with_message("Invalid argument passing.")
            }
            let argument = &args[1];
            match parse_command(argument) {
                Some(command) => match command {
                    Command::Status => {
                        match execute_status_command(&config) {
                            Ok(_) => {}
                            Err(err) => exit_with_message(&err.to_string()),
                        };
                    }
                    Command::Stage => {
                        match execute_stage_command(&config) {
                            Ok(_) => {}
                            Err(err) => exit_with_message(&err.to_string()),
                        };
                    }
                    Command::Config => {
                        execute_config_command();
                    }
                },
                None => exit_with_message("Invalid Command."),
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

fn execute_status_command(config: &Config) -> std::io::Result<()> {
    println!("Here is the current status: ");
    match create_writing_list(config) {
        Ok(writings) => print_writing_list(writings),
        Err(_) => exit_with_message("Couldn't print the writing list!"),
    }
    Ok(())
}

fn execute_stage_command(config: &Config) -> std::io::Result<()> {
    let writing_list = create_writing_list(config).expect("Writing list could not be created");
    let selected_writing =
        select_draft_writing_from_list(&writing_list).expect("Writing is not selected");
    let asset_list = get_asset_list_of_writing(selected_writing, config)
        .expect("Asset List could not be created");

    match transfer_asset_files(config, &asset_list) {
        Ok(_) => match update_writing_content_and_transfer(config, selected_writing, &asset_list) {
            Ok(_) => {
                println!("Writing transferred successfully.");
                Ok(())
            }
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    }
}

fn execute_config_command() {
    exit_with_message("config command is called");
}
