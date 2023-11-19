use std::{
    env,
    fs::{self, File},
    io::Write,
};

type ConfigResult<T> = Result<T, String>;

fn main() {
    let validate_config = validate_config();
    if validate_config.is_ok() {
        println!("hoho")
    }
    // parse arguments if we have any
    // check if we have a config file
    // if there is no file, exit and print config command
    // if there is a config file, check its validity
    // if not valid, exit and print config command
}

fn validate_config() -> ConfigResult<()> {
    if let Ok(home) = env::var("HOME") {
        let config_path = format!("{}/.config/lazydraft/lazydraft.json", home);

        if fs::metadata(&config_path).is_ok() {
            println!("Config file exists");
            return Ok(());
        }

        if let Some(parent) = std::path::Path::new(&config_path).parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    return Err(format!("Failed to create directory: {}", err));
                }
            }
        }

        match File::create(config_path) {
            Ok(mut file) => {
                println!("Config file is created successfully.");
                file.write_all(b"{}")
                    .expect("Failed to populate the empty onfig file");
                Ok(())
            }
            Err(e) => Err(format!("Failed to create config file: {}", e)),
        }
    } else {
        Err(String::from("Home environment variable not set"))
    }
}
