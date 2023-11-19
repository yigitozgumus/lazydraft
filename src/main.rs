use std::{
    env, fmt,
    fs::{self, File},
    io::{BufReader, Write},
};

use serde::{Deserialize, Serialize};

type ConfigResult<T> = Result<T, String>;

#[derive(Serialize, Deserialize)]
struct ConfigStruct {
    source_dir: String,
    source_asset_dir: String,
    target_dir: String,
    target_asset_dir: String,
    target_asset_prefix: String,
    yaml_asset_prefix: String,
}

impl fmt::Display for ConfigStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\nConfigStruct {{")?;
        writeln!(f, "    source_dir: {}", self.source_dir)?;
        writeln!(f, "    source_asset_dir: {}", self.source_asset_dir)?;
        writeln!(f, "    target_dir: {}", self.target_dir)?;
        writeln!(f, "    target_asset_dir: {}", self.target_asset_dir)?;
        writeln!(f, "    target_asset_prefix: {}", self.target_asset_prefix)?;
        writeln!(f, "    yaml_asset_prefix: {}", self.yaml_asset_prefix)?;
        write!(f, "}}")
    }
}

fn main() {
    match validate_config() {
        Ok(config) => {
            println!("Config is loaded: {}", config);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn validate_config() -> ConfigResult<ConfigStruct> {
    if let Ok(home) = env::var("HOME") {
        let config_path = format!("{}/.config/lazydraft/lazydraft.json", home);

        if fs::metadata(&config_path).is_ok() {
            // Read the JSON structure from the file
            let file = File::open(&config_path)
                .map_err(|err| format!("Failed to open a config file: {}", err))?;

            let reader = BufReader::new(file);

            let config: ConfigStruct = serde_json::from_reader(reader)
                .map_err(|e| format!("Failed to deserialize JSON: {}", e))?;

            return Ok(config);
        }

        if let Some(parent) = std::path::Path::new(&config_path).parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    return Err(format!("Failed to create directory: {}", err));
                }
            }
        }

        match File::create(&config_path) {
            Ok(mut file) => {
                let empty_config = ConfigStruct {
                    source_dir: String::new(),
                    source_asset_dir: String::new(),
                    target_dir: String::new(),
                    target_asset_dir: String::new(),
                    target_asset_prefix: String::new(),
                    yaml_asset_prefix: String::new(),
                };

                // Serialize the updated JSON structure
                let serialized_empty_config = match serde_json::to_string_pretty(&empty_config) {
                    Ok(content) => content,
                    Err(err) => return Err(format!("Failed to serialize JSON: {}", err)),
                };
                file.write_all(serialized_empty_config.as_bytes())
                    .map_err(|e| format!("Failed to initialize the config: {}", e))?;

                println!("Config file is created successfully at {}. Update the config and run the command again", config_path);

                std::process::exit(1);
            }
            Err(e) => Err(format!("Failed to create config file: {}", e)),
        }
    } else {
        Err(String::from("Home environment variable not set"))
    }
}
