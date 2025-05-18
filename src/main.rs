use asset::{get_asset_list_of_writing, transfer_asset_files};
use command::{parse_command, Command, StageOptions};
use config::{validate_config, Config};
use std::env;
use std::path::Path;
use std::process::Command as ProcessCommand;
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
            let args: Vec<String> = env::args().collect();

            // Split args into command and flags
            let command_args: Vec<String> = args.iter().skip(1).cloned().collect();

            match parse_command(&command_args) {
                Some(command) => match command {
                    Command::Status => {
                        check_config_for_empty_fields(&config);
                        match execute_status_command(&config) {
                            Ok(_) => {}
                            Err(err) => exit_with_message(&err.to_string()),
                        };
                    }
                    Command::Stage(options) => {
                        check_config_for_empty_fields(&config);
                        match execute_stage_command(&config, options) {
                            Ok(_) => {}
                            Err(err) => exit_with_message(&err.to_string()),
                        };
                    }
                    Command::Config => {
                        execute_config_command(command_args[1..].to_vec());
                    }
                    Command::Info => {
                        execute_info_command();
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

fn check_config_for_empty_fields(config: &Config) {
    if let Some(config) = config.has_empty_fields() {
        let message = format!(
            "{} is empty, please update config. You can open config using lazydraft config",
            config
        );
        exit_with_message(message.as_str());
    }
}

fn execute_info_command() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        r#"
LazyDraft - Version {}

Available Commands:
  status       - Displays the current status of your drafts and writings.
  stage        - Stages drafts and transfers content to the target location.
                 Options:
                 --continuous: Enables continuous monitoring and staging.
  config       - Validates and manages configuration settings.

Documentation and Help:
  Visit https://github.com/yigitozgumus/lazydraft for more details.
"#,
        version
    );
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

pub fn execute_config_command(args: Vec<String>) {
    // Check for additional flags
    if args.contains(&"--edit".to_string()) {
        open_config_in_editor();
    } else if args.contains(&"--info".to_string()) {
        display_config_info();
    } else {
        println!(
            "The `config` command allows you to manage the configuration.\n\n\
            Usage:\n  --edit   Open the config file in your selected editor.\n  \
            --info   Display information about each configuration option.\n\n\
            Example:\n  lazydraft config --edit\n  lazydraft config --info"
        );
    }
}

fn open_config_in_editor() {
    if let Ok(home) = env::var("HOME") {
        let config_path = format!("{}/.config/lazydraft/lazydraft.toml", home);
        let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        let status = std::process::Command::new(editor)
            .arg(&config_path)
            .status()
            .expect("Failed to open file with editor");

        if status.success() {
            println!("Config edited successfully.");
        } else {
            eprintln!("Editor exited with an error.");
        }
    } else {
        eprintln!("HOME environment variable is not set.");
    }
}

fn display_config_info() {
    println!(
        r#"
Configuration Options:

  source_dir                Directory where source files are located.
  source_asset_dir          Directory where assets for the source are stored.
  target_dir                Directory where output files are generated.
  target_asset_dir          Directory where output assets are stored.
  target_asset_prefix       Prefix for asset links in the generated files.
  target_hero_image_prefix  Prefix for hero image links in the output.
  yaml_asset_prefix         Prefix for assets referenced in YAML frontmatter.
  sanitize_frontmatter      If true, removes empty fields from the frontmatter.
  auto_add_cover_img        Automatically adds a cover image to the frontmatter.
  auto_add_hero_img         Automatically adds a hero image to the frontmatter.
  remove_draft_on_stage     Sets the 'draft' flag to false when staging.
  add_date_prefix           Adds a date prefix to the file name.
  remove_wikilinks          Converts wiki-style links to plain markdown links.
  trim_tags                 Strips a specified prefix from tags in frontmatter.
  tag_prefix                The prefix to strip from tags when 'trim_tags' is enabled.
  use_mdx_format            If true, saves output files with the .mdx extension instead of .md.

Use `lazydraft config --edit` to modify these settings.
"#
    );
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
        .watch(
            Path::new(&config.source_dir.as_deref().unwrap_or_default()),
            RecursiveMode::Recursive,
        )
        .expect("Failed to start watching directory");

    println!(
        "Watching for changes in: {}",
        config.source_dir.as_deref().unwrap_or_default()
    );

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
