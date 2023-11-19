mod config;

use config::*;

fn main() {
    match validate_config() {
        Ok(config) => {
            if config.has_empty_fields() {
                println!("There is an empty field in config. Please update it.");
                std::process::exit(1);
            } else {
                println!("Config is loaded: {}", config);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
