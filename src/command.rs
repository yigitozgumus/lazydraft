#[derive(Debug)]
pub enum Command {
    Status,
    Stage(StageOptions),
    Config,
    Info,
}

#[derive(Debug, Default)]
pub struct StageOptions {
    pub continuous: bool,
}

pub fn parse_command(args: &[String]) -> Option<Command> {
    if args.is_empty() {
        return Some(Command::Info);
    }

    match args[0].as_str() {
        "status" => {
            if args.len() > 1 {
                return None; // status command doesn't accept flags
            }
            Some(Command::Status)
        }
        "stage" => {
            let mut options = StageOptions::default();
            // Parse flags for stage command
            for arg in args.iter().skip(1) {
                match arg.as_str() {
                    "--continuous" | "-c" => options.continuous = true,
                    _ => return None, // Invalid flag
                }
            }
            Some(Command::Stage(options))
        }
        "config" => Some(Command::Config),
        _ => None,
    }
}
