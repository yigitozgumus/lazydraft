pub enum Command {
    Status,
    Stage,
    Config,
}

pub fn parse_command(arg: &str) -> Option<Command> {
    match arg {
        "status" => Some(Command::Status),
        "stage" => Some(Command::Stage),
        "config" => Some(Command::Config),
        _ => None,
    }
}
