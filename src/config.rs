use std::env;
use serde::{Deserialize, Serialize};

pub type ConfigResult<T> = Result<T, String>;

// Helper function to expand tilde in paths
pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

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
    /// Get source directory with tilde expansion
    pub fn get_source_dir(&self) -> Option<String> {
        self.source_dir.as_ref().map(|s| expand_tilde(s))
    }
    
    /// Get source asset directory with tilde expansion
    pub fn get_source_asset_dir(&self) -> Option<String> {
        self.source_asset_dir.as_ref().map(|s| expand_tilde(s))
    }
    
    /// Get target directory with tilde expansion
    pub fn get_target_dir(&self) -> Option<String> {
        self.target_dir.as_ref().map(|s| expand_tilde(s))
    }
    
    /// Get target asset directory with tilde expansion
    pub fn get_target_asset_dir(&self) -> Option<String> {
        self.target_asset_dir.as_ref().map(|s| expand_tilde(s))
    }

    /// Check if any required fields are empty
    pub fn has_empty_fields(&self) -> Option<String> {
        if self.source_dir.as_ref().map_or(true, |s| s.is_empty()) {
            return Some("source_dir".to_string());
        }
        if self.source_asset_dir.as_ref().map_or(true, |s| s.is_empty()) {
            return Some("source_asset_dir".to_string());
        }
        if self.target_dir.as_ref().map_or(true, |s| s.is_empty()) {
            return Some("target_dir".to_string());
        }
        if self.target_asset_dir.as_ref().map_or(true, |s| s.is_empty()) {
            return Some("target_asset_dir".to_string());
        }
        if self.target_asset_prefix.as_ref().map_or(true, |s| s.is_empty()) {
            return Some("target_asset_prefix".to_string());
        }
        if self.yaml_asset_prefix.as_ref().map_or(true, |s| s.is_empty()) {
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
