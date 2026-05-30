use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use crate::{
    asset::Asset,
    cli,
    config::Config,
    frontmatter,
};
use chrono::NaiveDate;
use colored::*;
use dialoguer::Select;
use itertools::Itertools;

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Writing {
    pub path: String,
    pub title: String,
    pub is_draft: bool,
    pub publish_date: Option<NaiveDate>,
}

impl Writing {
    fn new(path: String, title: String, is_draft: bool, publish_date: &str) -> Self {
        let date = match NaiveDate::parse_from_str(publish_date, "%Y-%m-%d") {
            Ok(date) => Some(date),
            Err(_) => None,
        };
        Writing {
            path,
            title,
            is_draft,
            publish_date: date,
        }
    }
}

pub fn print_writing_list(writings: Vec<Writing>) {
    cli::section("Writings");
    println!(
        "{:<5} {:<12} {:<12} {:<30}",
        "#", "Status", "Publish Date", "Title"
    );
    cli::divider_with(65);

    let mut draft_count = 0;
    let mut published_count = 0;

    for (index, writing) in writings.iter().enumerate() {
        let status_colored = if writing.is_draft {
            draft_count += 1;
            "Draft".yellow()
        } else {
            published_count += 1;
            "Published".green()
        };
        let publish_date = writing
            .publish_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());
        println!(
            "{:<5} {:<12} {:<12} {:<30}",
            index + 1,
            status_colored,
            publish_date,
            writing.title
        );
    }
    cli::divider_with(65);
    cli::kv(
        "Summary",
        format!(
            "{} draft(s), {} published writing(s)",
            draft_count.to_string().yellow(),
            published_count.to_string().green()
        ),
    );
}

pub fn select_draft_writing_from_list(writings: &Vec<Writing>) -> Option<&Writing> {
    let draft_writings: Vec<&Writing> = writings
        .iter()
        .filter(|&writing| writing.is_draft)
        .collect();

    if draft_writings.is_empty() {
        cli::warn("No draft writings available");
        return None;
    }

    let items: Vec<String> = draft_writings
        .iter()
        .map(|writing| writing.title.clone())
        .collect();

    let selection = match Select::new()
        .with_prompt("Select a draft writing")
        .items(&items)
        .interact()
    {
        Ok(index) => Some(index),
        Err(_) => None,
    };
    match selection {
        Some(index) => Some(draft_writings[index]),
        None => None,
    }
}

pub fn update_writing_content_and_transfer(
    config: &Config,
    writing: &Writing,
    asset_list: &Vec<Asset>,
) -> io::Result<()> {
    if let Ok((frontmatter, markdown_content)) = read_markdown_file(&writing.path) {
        let mut modifiable_frontmatter = frontmatter.clone();

        if config.remove_draft_on_stage.unwrap_or(false) {
            modifiable_frontmatter["draft"] = serde_yaml::to_value(false).expect("disable draft");
        }
        if config.sanitize_frontmatter.unwrap_or(false) {
            frontmatter::remove_empty_values(&mut modifiable_frontmatter);
        }
        if config.auto_add_cover_img.unwrap_or(false) {
            frontmatter::add_cover_image(&mut modifiable_frontmatter, config, &asset_list);
        }
        if config.auto_add_hero_img.unwrap_or(false) {
            frontmatter::add_hero_image(&mut modifiable_frontmatter, config, &asset_list);
        }
        if config.trim_tags.unwrap_or(false) {
            frontmatter::strip_tags(
                &mut modifiable_frontmatter,
                &config.tag_prefix.as_deref().unwrap_or(""),
            );
        }
        let mut updated_content = frontmatter::change_image_formats(markdown_content, config);
        if config.remove_wikilinks.unwrap_or(false) {
            updated_content = frontmatter::strip_wikilinks(updated_content.to_string());
        }

        let writing_name = frontmatter::create_writing_name(&mut modifiable_frontmatter, config, &writing.path);

        let target_dir = config
            .get_target_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "target_dir should be set"))?;
        // Determine file extension based on config
        let file_name = if config.use_mdx_format.unwrap_or(false) {
            // Change extension to .mdx
            let stem = Path::new(&writing_name)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            format!("{}.mdx", stem)
        } else {
            writing_name.clone()
        };
        let target_file_name = Path::new(&target_dir).join(file_name);
        
        // Ensure target directory exists
        if let Some(parent_dir) = target_file_name.parent() {
            fs::create_dir_all(parent_dir)?;
        }
        
        let merged_content = format!(
            "---\n{}\n{}",
            serde_yaml::to_string(&modifiable_frontmatter)
                .expect("frontmatter format should be correct after modification"),
            updated_content
        );
        let mut new_file = File::create(target_file_name)?;
        new_file.write_all(merged_content.as_bytes())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Cannot read writing."))
    }
}



pub fn read_markdown_file(
    file_path: &String,
) -> Result<(serde_yaml::Value, String), Box<dyn std::error::Error>> {
    let markdown_content = fs::read_to_string(file_path)?;
    let mut lines = markdown_content.lines().peekable();

    let frontmatter: serde_yaml::Value = match lines.next() {
        Some("---") => {
            let yaml_lines: Vec<&str> = lines
                .peeking_take_while(|line| !line.starts_with("---"))
                .collect();
            serde_yaml::from_str(&yaml_lines.join("\n"))?
        }
        _ => serde_yaml::Value::Null,
    };
    let markdown_content = lines.collect_vec().join("\n");
    Ok((frontmatter, markdown_content))
}

pub fn create_writing_list(config: &Config) -> Result<Vec<Writing>, Box<dyn std::error::Error>> {
    let directory_path = config
        .get_source_dir()
        .ok_or("source dir should be set")?;
    let mut writings: Vec<Writing> = Vec::new();

    for entry in WalkDir::new(&directory_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |e| e == "md") {
            let entry_path = entry.clone().into_path();
            if let Ok((frontmatter, _)) =
                read_markdown_file(&entry_path.as_path().display().to_string())
            {
                let title = frontmatter["title"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                let is_draft = frontmatter["draft"].as_bool().unwrap_or(false);
                let publish_date = frontmatter["publishDate"].as_str().unwrap_or("");
                let writing_path = entry_path.as_path().display().to_string();
                let writing = Writing::new(writing_path, title, is_draft, publish_date);
                writings.push(writing);
            }
        }
    }
    writings.sort_by(|a, b| {
        a.is_draft
            .cmp(&b.is_draft)
            .reverse()
            .then_with(|| a.publish_date.cmp(&b.publish_date).reverse())
    });
    Ok(writings)
}
