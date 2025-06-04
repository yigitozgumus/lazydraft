use std::fmt;

#[derive(Debug, Clone)]
pub enum Command {
    Status,
    Stage(StageOptions),
    Config,
    Info,
    Project(ProjectCommand),
    Dashboard,
}

#[derive(Debug, Clone)]
pub enum ProjectCommand {
    List,
    Create { name: String, description: Option<String> },
    Switch { name: String },
    Delete { name: String },
    Info { name: Option<String> },
    Rename { old_name: String, new_name: String },
}

#[derive(Debug, Clone)]
pub struct StageOptions {
    pub continuous: bool,
    pub project: Option<String>,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Status => write!(f, "status"),
            Command::Stage(_) => write!(f, "stage"),
            Command::Config => write!(f, "config"),
            Command::Info => write!(f, "info"),
            Command::Project(cmd) => write!(f, "project {}", cmd),
            Command::Dashboard => write!(f, "dashboard"),
        }
    }
}

impl fmt::Display for ProjectCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectCommand::List => write!(f, "list"),
            ProjectCommand::Create { name, .. } => write!(f, "create {}", name),
            ProjectCommand::Switch { name } => write!(f, "switch {}", name),
            ProjectCommand::Delete { name } => write!(f, "delete {}", name),
            ProjectCommand::Info { name } => {
                if let Some(n) = name {
                    write!(f, "info {}", n)
                } else {
                    write!(f, "info")
                }
            }
            ProjectCommand::Rename { old_name, new_name } => {
                write!(f, "rename {} {}", old_name, new_name)
            }
        }
    }
}

pub fn parse_command(args: &[String]) -> Option<Command> {
    if args.is_empty() {
        return None;
    }

    match args[0].as_str() {
        "status" => {
            let _project = extract_project_flag(args);
            Some(Command::Status)
        }
        "stage" => {
            let continuous = args.contains(&"--continuous".to_string());
            let project = extract_project_flag(args);
            Some(Command::Stage(StageOptions { continuous, project }))
        }
        "config" => Some(Command::Config),
        "info" => Some(Command::Info),
        "project" => parse_project_command(&args[1..]),
        "dashboard" => Some(Command::Dashboard),
        _ => None,
    }
}

fn parse_project_command(args: &[String]) -> Option<Command> {
    if args.is_empty() {
        return Some(Command::Project(ProjectCommand::List));
    }

    match args[0].as_str() {
        "list" => Some(Command::Project(ProjectCommand::List)),
        "create" => {
            if args.len() < 2 {
                return None;
            }
            let name = args[1].clone();
            let description = if args.len() > 2 {
                Some(args[2..].join(" "))
            } else {
                None
            };
            Some(Command::Project(ProjectCommand::Create { name, description }))
        }
        "switch" => {
            if args.len() < 2 {
                return None;
            }
            let name = args[1].clone();
            Some(Command::Project(ProjectCommand::Switch { name }))
        }
        "delete" => {
            if args.len() < 2 {
                return None;
            }
            let name = args[1].clone();
            Some(Command::Project(ProjectCommand::Delete { name }))
        }
        "info" => {
            let name = if args.len() > 1 {
                Some(args[1].clone())
            } else {
                None
            };
            Some(Command::Project(ProjectCommand::Info { name }))
        }
        "rename" => {
            if args.len() < 3 {
                return None;
            }
            let old_name = args[1].clone();
            let new_name = args[2].clone();
            Some(Command::Project(ProjectCommand::Rename { old_name, new_name }))
        }
        _ => None,
    }
}

fn extract_project_flag(args: &[String]) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == "--project" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}
