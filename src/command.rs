pub enum Command {
    List,
    Stage,
    Config,
}

pub fn parse_command(arg: &str) -> Option<Command> {
    match arg {
        "list" => Some(Command::List),
        "stage" => Some(Command::Stage),
        "config" => Some(Command::Config),
        _ => None,
    }
}
