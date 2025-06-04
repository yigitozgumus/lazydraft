use asset::{get_asset_list_of_writing, transfer_asset_files};
use command::{parse_command, Command, StageOptions, ProjectCommand};
use config::{validate_config, get_project_manager, Config};
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
mod command;
mod config;
mod writing;

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
                        eprintln!("Error: {}", e);
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
                        eprintln!("Error: {}", e);
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
    println!(
        r#"
LazyDraft - Version {}

Available Commands:
  status       - Displays the current status of your drafts and writings.
  stage        - Stages drafts and transfers content to the target location.
                 Options:
                 --continuous: Enables continuous monitoring and staging.
                 --project <name>: Use specific project instead of active one.
  config       - Validates and manages configuration settings.
                 Options:
                 --edit: Open config file in editor.
                 --info: Display configuration help.
                 --project <name>: Edit specific project config.

Project Management:
  project list           - List all projects and show active project.
  project create <name>  - Create a new project with optional description.
  project switch <name>  - Switch to a different project.
  project delete <name>  - Delete a project (cannot delete active project).
  project info [name]    - Show project details (current project if no name).
  project rename <old> <new> - Rename a project.

Examples:
  lazydraft project create my-blog "Personal blog content"
  lazydraft project switch my-blog
  lazydraft status
  lazydraft stage --continuous

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
    // Extract project flag if present
    let project_name = extract_project_from_args(&args);
    
    // Check for additional flags
    if args.contains(&"--edit".to_string()) {
        open_config_in_editor(project_name);
    } else if args.contains(&"--info".to_string()) {
        display_config_info();
    } else {
        println!(
            "The `config` command allows you to manage the configuration.\n\n\
            Usage:\n  --edit   Open the config file in your selected editor.\n  \
            --info   Display information about each configuration option.\n  \
            --project <name>   Work with specific project config.\n\n\
            Examples:\n  lazydraft config --edit\n  \
            lazydraft config --edit --project my-blog\n  \
            lazydraft config --info"
        );
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
                        eprintln!("No active project set and no project specified. Use --project <name> or set an active project.");
                        return;
                    }
                }
            }
        };
        
        // Check if config file exists
        if !std::path::Path::new(&config_path).exists() {
            eprintln!("Config file does not exist: {}", config_path);
            if let Some(name) = &project_name {
                eprintln!("Project '{}' not found. Use 'lazydraft project list' to see available projects.", name);
            }
            return;
        }
        
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

fn execute_project_command(cmd: ProjectCommand) -> Result<(), String> {
    let project_manager = get_project_manager()?;
    
    match cmd {
        ProjectCommand::List => {
            let projects = project_manager.list_projects()?;
            let active_project = project_manager.get_active_project()?;
            
            if projects.is_empty() {
                println!("No projects found. Create one with 'lazydraft project create <name>'");
                return Ok(());
            }
            
            println!("LazyDraft Projects:\n");
            for project in projects {
                let is_active = active_project.as_ref() == Some(&project.name);
                let marker = if is_active { "●" } else { " " };
                
                let source = project.config.source_dir.as_deref().unwrap_or("not set");
                let target = project.config.target_dir.as_deref().unwrap_or("not set");
                
                println!("  {} {}", marker, project.name);
                if let Some(desc) = &project.description {
                    println!("    {}", desc);
                }
                println!("    {} → {}", source, target);
                if let Some(last_used) = &project.last_used {
                    println!("    Last used: {}", format_timestamp(last_used));
                }
                println!();
            }
            
            if let Some(active) = active_project {
                println!("Active project: {}", active);
            } else {
                println!("No active project set. Use 'lazydraft project switch <name>' to select one.");
            }
        }
        ProjectCommand::Create { name, description } => {
            let project = project_manager.create_project(&name, description)?;
            println!("Created project '{}'", project.name);
            
            // Set as active if it's the first project
            let projects = project_manager.list_projects()?;
            if projects.len() == 1 {
                project_manager.set_active_project(&name)?;
                println!("Set '{}' as active project", name);
            }
            
            println!("Configure it with 'lazydraft config --project {}'", name);
        }
        ProjectCommand::Switch { name } => {
            project_manager.set_active_project(&name)?;
            println!("Switched to project '{}'", name);
        }
        ProjectCommand::Delete { name } => {
            // Check if it's the active project
            if let Some(active) = project_manager.get_active_project()? {
                if active == name {
                    return Err("Cannot delete the active project. Switch to another project first.".to_string());
                }
            }
            
            project_manager.delete_project(&name)?;
            println!("Deleted project '{}'", name);
        }
        ProjectCommand::Info { name } => {
            let project_name = match name {
                Some(n) => n,
                None => project_manager.get_active_project()?.ok_or("No active project set")?,
            };
            
            let project = project_manager.load_project(&project_name)?;
            
            println!("Project: {}", project.name);
            if let Some(desc) = &project.description {
                println!("Description: {}", desc);
            }
            if let Some(created) = &project.created_at {
                println!("Created: {}", format_timestamp(created));
            }
            if let Some(last_used) = &project.last_used {
                println!("Last used: {}", format_timestamp(last_used));
            }
            
            println!("\nConfiguration:");
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
            
            println!("Renamed project '{}' to '{}'", old_name, new_name);
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
    println!("  Source: {}", config.source_dir.as_deref().unwrap_or("not set"));
    println!("  Target: {}", config.target_dir.as_deref().unwrap_or("not set"));
    println!("  Source Assets: {}", config.source_asset_dir.as_deref().unwrap_or("not set"));
    println!("  Target Assets: {}", config.target_asset_dir.as_deref().unwrap_or("not set"));
    
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
        println!("  Features: {}", features.join(", "));
    }
}
