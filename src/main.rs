use command::{parse_command, Command};
use project::validate_config;

mod asset;
mod cli;
mod command;
mod commands;
mod config;
mod dashboard;
mod frontmatter;
mod project;
mod tui;
mod views;
mod writing;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command_args: Vec<String> = args.iter().skip(1).cloned().collect();

    match parse_command(&command_args) {
        Some(command) => match command {
            Command::Status => dispatch_status(),
            Command::Stage(options) => dispatch_stage(options),
            Command::Config => commands::execute_config_command(command_args),
            Command::Info => commands::execute_info_command(),
            Command::Project(cmd) => dispatch_project(cmd),
            Command::Dashboard => dispatch_dashboard(),
        },
        None => {
            if command_args.is_empty() {
                commands::execute_info_command();
            } else {
                commands::exit_with_message("Invalid Command. Use 'lazydraft info' for help.");
            }
        }
    }
}

fn dispatch_status() {
    match validate_config() {
        Ok(config) => {
            commands::check_config_for_empty_fields(&config);
            if let Err(err) = commands::execute_status_command(&config) {
                commands::exit_with_message(&err.to_string());
            }
        }
        Err(e) => {
            cli::error(&format!("Error: {}", e));
            std::process::exit(1);
        }
    }
}

fn dispatch_stage(options: command::StageOptions) {
    match validate_config() {
        Ok(config) => {
            commands::check_config_for_empty_fields(&config);
            if let Err(err) = commands::execute_stage_command(&config, options) {
                commands::exit_with_message(&err.to_string());
            }
        }
        Err(e) => {
            cli::error(&format!("Error: {}", e));
            std::process::exit(1);
        }
    }
}

fn dispatch_project(cmd: command::ProjectCommand) {
    if let Err(err) = commands::execute_project_command(cmd) {
        commands::exit_with_message(&err);
    }
}

fn dispatch_dashboard() {
    match dashboard::run_dashboard() {
        Ok(_) => {}
        Err(err) => commands::exit_with_message(&format!("Dashboard error: {}", err)),
    }
}
