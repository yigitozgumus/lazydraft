use std::path::Path;

use regex::Regex;
use serde_yaml::Value;

use crate::asset::Asset;
use crate::config::{Config, HeroImage, Image};

/// Strip a prefix from all tags in frontmatter
pub fn strip_tags(frontmatter: &mut Value, tag_prefix: &str) {
    if let Some(tags) = frontmatter.get_mut("tags") {
        if let Some(tag_list) = tags.as_sequence_mut() {
            for tag in tag_list.iter_mut() {
                if let Some(tag_str) = tag.as_str() {
                    if let Some(stripped) = tag_str.strip_prefix(tag_prefix) {
                        *tag = Value::String(stripped.to_string());
                    }
                }
            }
        }
    }
}

/// Remove null/empty values from YAML frontmatter
pub fn remove_empty_values(value: &mut Value) {
    match value {
        Value::Mapping(mapping) => {
            let keys: Vec<_> = mapping
                .iter()
                .filter_map(|(k, v)| if v.is_null() { Some(k.clone()) } else { None })
                .collect();

            for key in keys {
                mapping.remove(&key);
            }

            for (_, v) in mapping.iter_mut() {
                remove_empty_values(v);
            }
        }
        Value::Sequence(seq) => {
            seq.iter_mut().for_each(|v| remove_empty_values(v));
        }
        _ => {}
    }
}

/// Add cover image to frontmatter from matching asset
pub fn add_cover_image(frontmatter: &mut Value, config: &Config, asset_list: &[Asset]) {
    let asset_prefix = frontmatter["assetPrefix"].as_str().unwrap_or("");
    if asset_prefix.is_empty() {
        return;
    }
    let property_to_check = String::from(asset_prefix) + "-header";
    let matching_assets: Vec<&Asset> = asset_list
        .iter()
        .filter(|asset| asset.asset_path.contains(&property_to_check))
        .collect();
    if !matching_assets.is_empty() {
        let target_prefix = config
            .target_asset_prefix
            .as_ref()
            .expect("target asset prefix should be set");
        let header_name = Path::new(
            matching_assets
                .first()
                .expect("Header asset must exist")
                .asset_path
                .as_str(),
        )
        .file_name()
        .expect("Header asset name should be valid");
        let cover_img = Image {
            path: Path::new(target_prefix)
                .join(header_name)
                .as_path()
                .display()
                .to_string(),
            alt: "Cover Image".to_string(),
        };
        frontmatter["image"] = serde_yaml::to_value(&cover_img).expect("Cover Image format should match");
    }
}

/// Add hero image to frontmatter from matching asset
pub fn add_hero_image(frontmatter: &mut Value, config: &Config, asset_list: &[Asset]) {
    let asset_prefix = frontmatter["assetPrefix"].as_str().unwrap_or("");
    if asset_prefix.is_empty() {
        return;
    }
    let property_to_check = String::from(asset_prefix) + "-header";
    let matching_assets: Vec<&Asset> = asset_list
        .iter()
        .filter(|asset| asset.asset_path.contains(&property_to_check))
        .collect();
    if !matching_assets.is_empty() {
        let target_prefix = config
            .target_hero_image_prefix
            .as_ref()
            .expect("target hero image prefix should be set");
        let header_name = Path::new(
            matching_assets
                .first()
                .expect("Header asset must exist")
                .asset_path
                .as_str(),
        )
        .file_name()
        .expect("Header asset name should be valid");
        let hero_img = HeroImage {
            path: Path::new(target_prefix)
                .join(header_name)
                .as_path()
                .display()
                .to_string(),
            alt: "Social Cover Image".to_string(),
        };
        frontmatter["heroImage"] = serde_yaml::to_value(&hero_img).expect("Hero image format should match");
    }
}

/// Convert [[wikilink]] image references to standard markdown image syntax
pub fn change_image_formats(content: String, config: &Config) -> String {
    let pattern = Regex::new(r"!\[\[(.*?)\]\]").expect("Failed to create image wikilink regex");
    let target_prefix = &config.target_asset_prefix;
    pattern
        .replace_all(&content, |caps: &regex::Captures| {
            if let Some(link) = caps.get(1) {
                format!(
                    "![]({}/{})",
                    target_prefix.as_deref().unwrap_or(""),
                    link.as_str()
                )
            } else {
                caps.get(0).unwrap().as_str().to_string()
            }
        })
        .to_string()
}

/// Convert all [[wikilinks]] to plain text
pub fn strip_wikilinks(content: String) -> String {
    let pattern = Regex::new(r"\[\[(.*?)\]\]").expect("Failed to create wikilink regex");
    pattern
        .replace_all(&content, |caps: &regex::Captures| {
            if let Some(link) = caps.get(1) {
                link.as_str().to_string()
            } else {
                caps.get(0).unwrap().as_str().to_string()
            }
        })
        .to_string()
}

/// Build the output file name based on frontmatter date prefix setting
pub fn create_writing_name(frontmatter: &mut Value, config: &Config, writing_path: &str) -> String {
    let mut writing_name = Path::new(writing_path)
        .file_name()
        .expect("Could not parse writing name")
        .to_str()
        .expect("Parsed writing name shouldn't be empty")
        .to_string();

    let publish_date = frontmatter["publishDate"].as_str().unwrap_or("");

    if config.add_date_prefix.unwrap_or(false) && !publish_date.is_empty() {
        writing_name = format!("{}-{}", publish_date, writing_name);
    }
    writing_name
}
