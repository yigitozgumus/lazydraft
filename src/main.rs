use asset::{get_asset_list_of_writing, transfer_asset_files};
use command::{parse_command, Command, StageOptions};
use config::{validate_config, Config};
use std::env;
use std::path::Path;
use writing::{
    create_writing_list, print_writing_list, select_draft_writing_from_list,
    update_writing_content_and_transfer,
};

mod asset;
mod command;
mod config;
mod writing;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    match validate_config() {
        Ok(config) => {
            if config.has_empty_fields() {
                exit_with_message("There is an empty field in config. Please update it")
            }
            let args: Vec<String> = env::args().collect();

            // Split args into command and flags
            let command_args: Vec<String> = args.iter().skip(1).cloned().collect();

            match parse_command(&command_args) {
                Some(command) => match command {
                    Command::Status => {
                        match execute_status_command(&config) {
                            Ok(_) => {}
                            Err(err) => exit_with_message(&err.to_string()),
                        };
                    }
                    Command::Stage(options) => {
                        match execute_stage_command(&config, options) {
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

fn execute_config_command() {
    exit_with_message("config command is called");
}

fn execute_stage_command(config: &Config, options: StageOptions) -> std::io::Result<()> {
    if options.continuous {
        execute_continuous_stage(config)
    } else {
        execute_single_stage(config)
    }
}

fn execute_single_stage(config: &Config) -> std::io::Result<()> {
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

fn execute_continuous_stage(config: &Config) -> std::io::Result<()> {
    println!("Starting continuous staging mode...");

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        tx,
        NotifyConfig::default().with_poll_interval(Duration::from_secs(2)),
    )
    .expect("Failed to create file watcher");

    // Watch the source directory
    watcher
        .watch(Path::new(&config.source_dir), RecursiveMode::Recursive)
        .expect("Failed to start watching directory");

    println!("Watching for changes in: {}", config.source_dir);

    let config = config.clone();

    loop {
        match rx.recv() {
            Ok(event) => match event {
                Ok(event) => {
                    if event.kind.is_modify() {
                        println!("\nChange detected, running stage process...");

                        match create_writing_list(&config) {
                            Ok(writing_list) => {
                                if let Some(modified_writing) = writing_list.iter().find(|w| {
                                    event
                                        .paths
                                        .iter()
                                        .any(|p| p.to_string_lossy().contains(&w.path))
                                }) {
                                    if modified_writing.is_draft {
                                        match get_asset_list_of_writing(modified_writing, &config) {
                                            Ok(asset_list) => {
                                                match transfer_asset_files(&config, &asset_list) {
                                                        Ok(_) => {
                                                            match update_writing_content_and_transfer(&config, modified_writing, &asset_list) {
                                                                Ok(_) => println!("Successfully staged changes for: {}", modified_writing.title),
                                                                Err(e) => eprintln!("Error updating content: {}", e),
                                                            }
                                                        }
                                                        Err(e) => eprintln!("Error transferring assets: {}", e),
                                                    }
                                            }
                                            Err(e) => eprintln!("Error getting asset list: {}", e),
                                        }
                                    }
                                }
                            }
                            Err(e) => eprintln!("Error creating writing list: {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {}", e),
            },
            Err(e) => eprintln!("Watch error: {}", e),
        }
    }
}
