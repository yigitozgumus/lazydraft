use asset::{get_asset_list_of_writing, transfer_asset_files};
use command::{parse_command, Command, ProjectCommand, StageOptions};
use config::{get_project_manager, validate_config, Config};
use std::env;
use std::path::Path;
use writing::{
    create_writing_list, print_writing_list, select_draft_writing_from_list,
    update_writing_content_and_transfer,
};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

mod asset;
mod cli;
mod command;
mod config;
mod writing;
mod dashboard;
mod tui;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command_args: Vec<String> = args.iter().skip(1).cloned().collect();

    match parse_command(&command_args) {
        Some(command) => match command {
            Command::Status => {
                match validate_config() {
                    Ok(config) => {
                        check_config_for_empty_fields(&config);
                        match execute_status_command(&config) {
                            Ok(_) => {}
                            Err(err) => exit_with_message(&err.to_string()),
                        };
                    }
                    Err(e) => {
                        cli::error(&format!("Error: {}", e));
                        std::process::exit(1);
                    }
                }
            }
            Command::Stage(options) => {
                match validate_config() {
                    Ok(config) => {
                        check_config_for_empty_fields(&config);
                        match execute_stage_command(&config, options) {
                            Ok(_) => {}
                            Err(err) => exit_with_message(&err.to_string()),
                        };
                    }
                    Err(e) => {
                        cli::error(&format!("Error: {}", e));
                        std::process::exit(1);
                    }
                }
            }
            Command::Config => {
                execute_config_command(command_args);
            }
            Command::Info => {
                execute_info_command();
            }
            Command::Project(project_cmd) => {
                match execute_project_command(project_cmd) {
                    Ok(_) => {}
                    Err(err) => exit_with_message(&err),
                }
            }
            Command::Dashboard => {
                match dashboard::run_dashboard() {
                    Ok(_) => {}
                    Err(err) => exit_with_message(&format!("Dashboard error: {}", err)),
                }
            }
        },
        None => {
            if command_args.is_empty() {
                execute_info_command();
            } else {
                exit_with_message("Invalid Command. Use 'lazydraft info' for help.");
            }
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
    cli::header(
        &format!("LazyDraft {}", version),
        Some("Draft staging and project workflows"),
    );
    cli::blank_line();
    cli::section("Commands");
    cli::list_item("status      Show drafts and published writings");
    cli::list_item("stage       Stage drafts and transfer content");
    cli::list_item("config      Edit or inspect configuration");
    cli::list_item("dashboard   Launch the interactive TUI");
    cli::blank_line();
    cli::section("Stage Options");
    cli::list_item("--continuous   Watch source folder and stage on changes");
    cli::list_item("--project <name>  Use a specific project");
    cli::blank_line();
    cli::section("Project Management");
    cli::list_item("project list           List projects and show active");
    cli::list_item("project create <name>  Create a project");
    cli::list_item("project switch <name>  Switch active project");
    cli::list_item("project delete <name>  Delete a project (not active)");
    cli::list_item("project info [name]    Show project details");
    cli::list_item("project rename <old> <new>  Rename a project");
    cli::blank_line();
    cli::section("Examples");
    cli::list_item("lazydraft dashboard");
    cli::list_item("lazydraft project create my-blog \"Personal blog content\"");
    cli::list_item("lazydraft project switch my-blog");
    cli::list_item("lazydraft status");
    cli::list_item("lazydraft stage --continuous");
    cli::blank_line();
    cli::section("Documentation");
    cli::list_item("https://github.com/yigitozgumus/lazydraft");
}

fn exit_with_message(message: &str) {
    cli::error(message);
    std::process::exit(1);
}

fn execute_status_command(config: &Config) -> std::io::Result<()> {
    match create_writing_list(config) {
        Ok(writings) => print_writing_list(writings),
        Err(_) => exit_with_message("Couldn't print the writing list!"),
    }
    Ok(())
}

pub fn execute_config_command(args: Vec<String>) {
    // Extract project flag if present
    let project_name = extract_project_from_args(&args);
    
    // Check for additional flags
    if args.contains(&"--edit".to_string()) {
        open_config_in_editor(project_name);
    } else if args.contains(&"--info".to_string()) {
        display_config_info();
    } else {
        cli::section("Config Command");
        cli::list_item("--edit      Open the config file in your editor");
        cli::list_item("--info      Display details about each setting");
        cli::list_item("--project <name>  Target a specific project");
        cli::blank_line();
        cli::section("Examples");
        cli::list_item("lazydraft config --edit");
        cli::list_item("lazydraft config --edit --project my-blog");
        cli::list_item("lazydraft config --info");
    }
}

fn extract_project_from_args(args: &[String]) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == "--project" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn open_config_in_editor(project_name: Option<String>) {
    if let Ok(home) = env::var("HOME") {
        let config_path = match project_name.as_ref() {
            Some(name) => {
                format!("{}/.config/lazydraft/projects/{}.toml", home, name)
            }
            None => {
                // Try to get active project
                match get_project_manager().and_then(|pm| pm.get_active_project()) {
                    Ok(Some(active_name)) => {
                        format!("{}/.config/lazydraft/projects/{}.toml", home, active_name)
                    }
                    _ => {
                        cli::warn("No active project set and no project specified. Use --project <name> or set an active project.");
                        return;
                    }
                }
            }
        };
        
        // Check if config file exists
        if !std::path::Path::new(&config_path).exists() {
            cli::warn(&format!("Config file does not exist: {}", config_path));
            if let Some(name) = &project_name {
                cli::warn(&format!(
                    "Project '{}' not found. Use 'lazydraft project list' to see available projects.",
                    name
                ));
            }
            return;
        }
        
        let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        let status = std::process::Command::new(editor)
            .arg(&config_path)
            .status()
            .expect("Failed to open file with editor");

        if status.success() {
            cli::success("Config edited successfully.");
        } else {
            cli::error("Editor exited with an error.");
        }
    } else {
        cli::error("HOME environment variable is not set.");
    }
}

fn display_config_info() {
    cli::section("Configuration Options");
    cli::kv("source_dir", "Directory where source files are located.");
    cli::kv("source_asset_dir", "Directory where assets for the source are stored.");
    cli::kv("target_dir", "Directory where output files are generated.");
    cli::kv("target_asset_dir", "Directory where output assets are stored.");
    cli::kv("target_asset_prefix", "Prefix for asset links in the generated files.");
    cli::kv(
        "target_hero_image_prefix",
        "Prefix for hero image links in the output.",
    );
    cli::kv("yaml_asset_prefix", "Prefix for assets referenced in YAML frontmatter.");
    cli::kv(
        "sanitize_frontmatter",
        "If true, removes empty fields from the frontmatter.",
    );
    cli::kv(
        "auto_add_cover_img",
        "Automatically adds a cover image to the frontmatter.",
    );
    cli::kv(
        "auto_add_hero_img",
        "Automatically adds a hero image to the frontmatter.",
    );
    cli::kv(
        "remove_draft_on_stage",
        "Sets the 'draft' flag to false when staging.",
    );
    cli::kv("add_date_prefix", "Adds a date prefix to the file name.");
    cli::kv(
        "remove_wikilinks",
        "Converts wiki-style links to plain markdown links.",
    );
    cli::kv(
        "trim_tags",
        "Strips a specified prefix from tags in frontmatter.",
    );
    cli::kv(
        "tag_prefix",
        "The prefix to strip from tags when 'trim_tags' is enabled.",
    );
    cli::kv(
        "use_mdx_format",
        "If true, saves output files with the .mdx extension instead of .md.",
    );
    cli::blank_line();
    cli::info("Use `lazydraft config --edit` to modify these settings.");
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
                cli::success("Writing transferred successfully.");
                Ok(())
            }
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    }
}

fn execute_continuous_stage(config: &Config) -> std::io::Result<()> {
    cli::info("Starting continuous staging mode...");

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        tx,
        NotifyConfig::default().with_poll_interval(Duration::from_secs(2)),
    )
    .expect("Failed to create file watcher");

    // Watch the source directory
    watcher
        .watch(
            Path::new(&config.get_source_dir().unwrap_or_default()),
            RecursiveMode::Recursive,
        )
        .expect("Failed to start watching directory");

    cli::info(&format!(
        "Watching for changes in: {}",
        config.get_source_dir().unwrap_or_default()
    ));

    let config = config.clone();

    loop {
        match rx.recv() {
            Ok(event) => match event {
                Ok(event) => {
                    if event.kind.is_modify() {
                        cli::info("Change detected, running stage process...");

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
                                                                Ok(_) => cli::success(&format!(
                                                                    "Staged changes for: {}",
                                                                    modified_writing.title
                                                                )),
                                                                Err(e) => cli::error(&format!(
                                                                    "Error updating content: {}",
                                                                    e
                                                                )),
                                                            }
                                                        }
                                                        Err(e) => cli::error(&format!(
                                                            "Error transferring assets: {}",
                                                            e
                                                        )),
                                                    }
                                            }
                                            Err(e) => cli::error(&format!(
                                                "Error getting asset list: {}",
                                                e
                                            )),
                                        }
                                    }
                                }
                            }
                            Err(e) => cli::error(&format!(
                                "Error creating writing list: {}",
                                e
                            )),
                        }
                    }
                }
                Err(e) => cli::error(&format!("Watch error: {}", e)),
            },
            Err(e) => cli::error(&format!("Watch error: {}", e)),
        }
    }
}

fn execute_project_command(cmd: ProjectCommand) -> Result<(), String> {
    let project_manager = get_project_manager()?;
    
    match cmd {
        ProjectCommand::List => {
            let projects = project_manager.list_projects()?;
            let active_project = project_manager.get_active_project()?;
            
            if projects.is_empty() {
                cli::warn("No projects found. Create one with 'lazydraft project create <name>'");
                return Ok(());
            }
            cli::section("Projects");
            for project in projects {
                let is_active = active_project.as_ref() == Some(&project.name);
                let marker = if is_active { "*" } else { " " };
                
                let source = project.config.get_source_dir().unwrap_or_else(|| "not set".to_string());
                let target = project.config.get_target_dir().unwrap_or_else(|| "not set".to_string());
                
                cli::list_item(&format!("{} {}", marker, project.name));
                if let Some(desc) = &project.description {
                    cli::kv("Description", desc);
                }
                cli::kv("Paths", format!("{} -> {}", source, target));
                if let Some(last_used) = &project.last_used {
                    cli::kv("Last used", format_timestamp(last_used));
                }
                cli::blank_line();
            }
            
            if let Some(active) = active_project {
                cli::info(&format!("Active project: {}", active));
            } else {
                cli::warn("No active project set. Use 'lazydraft project switch <name>' to select one.");
            }
        }
        ProjectCommand::Create { name, description } => {
            let project = project_manager.create_project(&name, description)?;
            cli::success(&format!("Created project '{}'", project.name));
            
            // Set as active if it's the first project
            let projects = project_manager.list_projects()?;
            if projects.len() == 1 {
                project_manager.set_active_project(&name)?;
                cli::success(&format!("Set '{}' as active project", name));
            }
            cli::info(&format!(
                "Configure it with 'lazydraft config --project {}'",
                name
            ));
        }
        ProjectCommand::Switch { name } => {
            project_manager.set_active_project(&name)?;
            cli::success(&format!("Switched to project '{}'", name));
        }
        ProjectCommand::Delete { name } => {
            // Check if it's the active project
            if let Some(active) = project_manager.get_active_project()? {
                if active == name {
                    return Err("Cannot delete the active project. Switch to another project first.".to_string());
                }
            }
            
            project_manager.delete_project(&name)?;
            cli::success(&format!("Deleted project '{}'", name));
        }
        ProjectCommand::Info { name } => {
            let project_name = match name {
                Some(n) => n,
                None => project_manager.get_active_project()?.ok_or("No active project set")?,
            };
            
            let project = project_manager.load_project(&project_name)?;
            
            cli::section(&format!("Project: {}", project.name));
            if let Some(desc) = &project.description {
                cli::kv("Description", desc);
            }
            if let Some(created) = &project.created_at {
                cli::kv("Created", format_timestamp(created));
            }
            if let Some(last_used) = &project.last_used {
                cli::kv("Last used", format_timestamp(last_used));
            }
            cli::blank_line();
            cli::section("Configuration");
            print_config_summary(&project.config);
        }
        ProjectCommand::Rename { old_name, new_name } => {
            let mut project = project_manager.load_project(&old_name)?;
            project.name = new_name.clone();
            
            // Save with new name and delete old
            project_manager.save_project(&project)?;
            project_manager.delete_project(&old_name)?;
            
            // Update active project if needed
            if let Some(active) = project_manager.get_active_project()? {
                if active == old_name {
                    project_manager.set_active_project(&new_name)?;
                }
            }
            
            cli::success(&format!("Renamed project '{}' to '{}'", old_name, new_name));
        }
    }
    
    Ok(())
}

fn format_timestamp(timestamp: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(timestamp) {
        Ok(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        Err(_) => timestamp.to_string(),
    }
}

fn print_config_summary(config: &Config) {
    cli::kv(
        "Source",
        config.get_source_dir().unwrap_or_else(|| "not set".to_string()),
    );
    cli::kv(
        "Target",
        config.get_target_dir().unwrap_or_else(|| "not set".to_string()),
    );
    cli::kv(
        "Source Assets",
        config.get_source_asset_dir().unwrap_or_else(|| "not set".to_string()),
    );
    cli::kv(
        "Target Assets",
        config.get_target_asset_dir().unwrap_or_else(|| "not set".to_string()),
    );
    
    let mut features = Vec::new();
    if config.sanitize_frontmatter.unwrap_or(false) { features.push("sanitize frontmatter"); }
    if config.auto_add_cover_img.unwrap_or(false) { features.push("auto cover image"); }
    if config.auto_add_hero_img.unwrap_or(false) { features.push("auto hero image"); }
    if config.remove_draft_on_stage.unwrap_or(false) { features.push("remove draft on stage"); }
    if config.add_date_prefix.unwrap_or(false) { features.push("date prefix"); }
    if config.remove_wikilinks.unwrap_or(false) { features.push("remove wikilinks"); }
    if config.trim_tags.unwrap_or(false) { features.push("trim tags"); }
    if config.use_mdx_format.unwrap_or(false) { features.push("MDX format"); }
    
    if !features.is_empty() {
        cli::kv("Features", features.join(", "));
    }
}
