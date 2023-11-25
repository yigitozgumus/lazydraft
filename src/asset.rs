use std::{
    fs, io,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::writing::read_markdown_file;

use crate::{config::Config, writing::Writing};

#[derive(Debug)]
pub struct Asset {
    pub asset_path: String,
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
                Ok(_) => {}
                Err(err) => eprintln!("Error copying file: {}", err),
            }
        }
    }
    println!("Asset files are copied successfully.");
    Ok(())
}
