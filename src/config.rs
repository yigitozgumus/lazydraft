use std::fs;
use std::io::Write;
use std::{env, fmt, fs::File, io::BufReader};
use toml;

use serde::{Deserialize, Serialize};

pub type ConfigResult<T> = Result<T, String>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub source_dir: Option<String>,
    #[serde(default)]
    pub source_asset_dir: Option<String>,
    #[serde(default)]
    pub target_dir: Option<String>,
    #[serde(default)]
    pub target_asset_dir: Option<String>,
    #[serde(default)]
    pub target_asset_prefix: Option<String>,
    #[serde(default)]
    pub target_hero_image_prefix: Option<String>,
    #[serde(default)]
    pub yaml_asset_prefix: Option<String>,
    #[serde(default)]
    pub sanitize_frontmatter: Option<bool>,
    #[serde(default)]
    pub auto_add_cover_img: Option<bool>,
    #[serde(default)]
    pub auto_add_hero_img: Option<bool>,
    #[serde(default)]
    pub remove_draft_on_stage: Option<bool>,
    #[serde(default)]
    pub add_date_prefix: Option<bool>,
    #[serde(default)]
    pub remove_wikilinks: Option<bool>,
    #[serde(default)]
    pub trim_tags: Option<bool>,
    #[serde(default)]
    pub tag_prefix: Option<String>,
    #[serde(default)]
    pub use_mdx_format: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub path: String,
    pub alt: String,
}

#[derive(Serialize, Deserialize)]
pub struct HeroImage {
    pub path: String,
    pub alt: String,
}

impl Config {
    // Method to check if any fields are empty
    pub fn has_empty_fields(&self) -> Option<String> {
        if self.source_dir.as_ref().map_or(true, |s| s.is_empty()) {
            return Some("source_dir".to_string());
        }
        if self
            .source_asset_dir
            .as_ref()
            .map_or(true, |s| s.is_empty())
        {
            return Some("source_asset_dir".to_string());
        }
        if self.target_dir.as_ref().map_or(true, |s| s.is_empty()) {
            return Some("target_dir".to_string());
        }
        if self
            .target_asset_dir
            .as_ref()
            .map_or(true, |s| s.is_empty())
        {
            return Some("target_asset_dir".to_string());
        }
        if self
            .target_asset_prefix
            .as_ref()
            .map_or(true, |s| s.is_empty())
        {
            return Some("target_asset_dir".to_string());
        }
        if self
            .yaml_asset_prefix
            .as_ref()
            .map_or(true, |s| s.is_empty())
        {
            return Some("yaml_asset_prefix".to_string());
        }
        if self.trim_tags.unwrap_or(false)
            && self.tag_prefix.as_ref().map_or(true, |s| s.is_empty())
        {
            return Some("tag_prefix".to_string());
        }
        None
    }
}

pub fn validate_config() -> ConfigResult<Config> {
    if let Ok(home) = env::var("HOME") {
        let config_dir = format!("{}/.config/lazydraft", home);
        let toml_path = format!("{}/lazydraft.toml", config_dir);
        let json_path = format!("{}/lazydraft.json", config_dir);

        // MIGRATION STEP: If TOML does not exist but JSON does, migrate
        if !fs::metadata(&toml_path).is_ok() && fs::metadata(&json_path).is_ok() {
            // Read JSON config
            let file = File::open(&json_path)
                .map_err(|err| format!("Failed to open JSON config for migration: {}", err))?;
            let reader = BufReader::new(file);
            let config: Config = match serde_json::from_reader(reader) {
                Ok(cfg) => cfg,
                Err(e) => {
                    return Err(format!(
                        "Failed to deserialize JSON during migration: {}",
                        e
                    ))
                }
            };
            // Write TOML config
            let serialized_toml = match toml::to_string_pretty(&config) {
                Ok(content) => content,
                Err(e) => return Err(format!("Failed to serialize TOML during migration: {}", e)),
            };
            if let Some(parent) = std::path::Path::new(&toml_path).parent() {
                if !parent.exists() {
                    if let Err(err) = fs::create_dir_all(parent) {
                        return Err(format!("Failed to create directory: {}", err));
                    }
                }
            }
            let mut file = File::create(&toml_path)
                .map_err(|e| format!("Failed to create TOML config during migration: {}", e))?;
            file.write_all(serialized_toml.as_bytes())
                .map_err(|e| format!("Failed to write TOML config during migration: {}", e))?;
            // Optionally, remove or rename the old JSON file
            let _ = fs::remove_file(&json_path);
            println!("Migrated configuration from lazydraft.json to lazydraft.toml");
        }

        if fs::metadata(&toml_path).is_ok() {
            // Read the TOML structure from the file
            let file = File::open(&toml_path)
                .map_err(|err| format!("Failed to open a config file: {}", err))?;

            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            use std::io::Read;
            reader
                .read_to_string(&mut contents)
                .map_err(|e| format!("Failed to read TOML: {}", e))?;
            let config: Config = toml::from_str(&contents)
                .map_err(|e| format!("Failed to deserialize TOML: {}", e))?;
            return Ok(config);
        }

        if let Some(parent) = std::path::Path::new(&toml_path).parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    return Err(format!("Failed to create directory: {}", err));
                }
            }
        }

        match File::create(&toml_path) {
            Ok(mut file) => {
                let empty_config = Config {
                    source_dir: None,
                    source_asset_dir: None,
                    target_dir: None,
                    target_asset_dir: None,
                    target_asset_prefix: None,
                    target_hero_image_prefix: None,
                    yaml_asset_prefix: None,
                    sanitize_frontmatter: Some(false),
                    auto_add_cover_img: Some(false),
                    auto_add_hero_img: Some(false),
                    remove_draft_on_stage: Some(false),
                    add_date_prefix: Some(false),
                    remove_wikilinks: Some(false),
                    trim_tags: Some(false),
                    tag_prefix: None,
                    use_mdx_format: Some(false),
                };

                // Serialize the updated TOML structure
                let serialized_empty_config = match toml::to_string_pretty(&empty_config) {
                    Ok(content) => content,
                    Err(err) => return Err(format!("Failed to serialize TOML: {}", err)),
                };
                file.write_all(serialized_empty_config.as_bytes())
                    .map_err(|e| format!("Failed to initialize the config: {}", e))?;

                println!("Config file is created successfully at {}", toml_path);

                Ok(empty_config)
            }
            Err(e) => Err(format!("Failed to create config file: {}", e)),
        }
    } else {
        Err(String::from("Home environment variable not set"))
    }
}
