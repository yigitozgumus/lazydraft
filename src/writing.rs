use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::{asset::Asset, config::Config};
use chrono::NaiveDate;
use dialoguer::Select;
use itertools::Itertools;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Writing {
    path: PathBuf,
    pub title: String,
    is_draft: bool,
    publish_date: Option<NaiveDate>,
}

impl Writing {
    fn new(path: PathBuf, title: String, is_draft: bool, publish_date: &str) -> Self {
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
    for (index, writing) in writings.iter().enumerate() {
        let draft = if writing.is_draft {
            "Draft"
        } else {
            "Published"
        };
        println!("{} - ({}) {} ", index + 1, draft, writing.title);
    }
}

pub fn get_asset_list_of_writing(writing: &Writing, config: &Config) -> Option<Vec<Asset>> {
    let (frontmatter, _) = read_markdown_file(&writing.path).unwrap();
    let prefix = &config.yaml_asset_prefix;
    let writing_prefix = frontmatter[prefix].as_str().unwrap();
    let mut asset_list: Vec<Asset> = Vec::new();
    for asset in WalkDir::new(&config.source_asset_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if asset.file_type().is_file() {
            let file_name = asset.file_name().to_string_lossy();
            if file_name.contains(writing_prefix) {
                let current = Asset {
                    asset_path: asset.into_path().display().to_string(),
                };
                asset_list.push(current);
            }
        }
    }
    if asset_list.is_empty() {
        None
    } else {
        Some(asset_list)
    }
}

pub fn transfer_asset_files(config: &Config, asset_list: Vec<Asset>) -> io::Result<()> {
    for asset in asset_list {
        let path = Path::new(&asset.asset_path);
        let file_name = path.file_name().unwrap();
        if path.is_file() {
            let destination_path = PathBuf::from(&config.target_asset_dir).join(file_name);
            match fs::copy(path, destination_path) {
                Ok(_) => println!("File copied successfully."),
                Err(err) => eprintln!("Error copying file: {}", err),
            }
        }
    }
    Ok(())
}

pub fn select_draft_writing_from_list(writings: &Vec<Writing>) -> Option<&Writing> {
    let draft_writings: Vec<&Writing> = writings
        .iter()
        .filter(|&writing| writing.is_draft)
        .collect();

    if draft_writings.is_empty() {
        println!("No draft writings available");
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

fn read_markdown_file(
    file_path: &PathBuf,
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
    let markdown_content: String = lines.collect();
    Ok((frontmatter, markdown_content))
}

pub fn create_writing_list(config: &Config) -> Result<Vec<Writing>, Box<dyn std::error::Error>> {
    let directory_path = &config.source_dir;
    let mut writings: Vec<Writing> = Vec::new();

    for entry in WalkDir::new(directory_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |e| e == "md") {
            let entry_path = entry.clone().into_path();
            if let Ok((frontmatter, _)) = read_markdown_file(&entry_path) {
                let title = frontmatter["title"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                let is_draft = frontmatter["draft"].as_bool().unwrap_or(false);
                let publish_date = frontmatter["publishDate"].as_str().unwrap_or("");

                let writing = Writing::new(entry.into_path(), title, is_draft, publish_date);
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
