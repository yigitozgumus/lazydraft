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

pub fn get_asset_list_of_writing(writing: &Writing, config: &Config) -> io::Result<Vec<Asset>> {
    let (frontmatter, _) = read_markdown_file(&writing.path).unwrap();
    let prefix = &config.yaml_asset_prefix.as_deref().unwrap_or_default();
    let writing_prefix = frontmatter[prefix].as_str().unwrap_or("");
    if writing_prefix.is_empty() {
        return Ok(Vec::new());
    }
    
    // Use expanded source asset directory
    let source_asset_dir = config.get_source_asset_dir().unwrap_or_default();
    if source_asset_dir.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut asset_list: Vec<Asset> = Vec::new();
    for asset in WalkDir::new(&source_asset_dir)
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
    Ok(asset_list)
}

pub fn transfer_asset_files(config: &Config, asset_list: &Vec<Asset>) -> io::Result<()> {
    if asset_list.is_empty() {
        return Ok(());
    }
    
    // Use expanded target asset directory
    let target_asset_dir = config.get_target_asset_dir().unwrap_or_default();
    if target_asset_dir.is_empty() {
        return Ok(());
    }
    
    // Ensure target asset directory exists
    fs::create_dir_all(&target_asset_dir)?;
    
    for asset in asset_list {
        let path = Path::new(&asset.asset_path);
        let file_name = path.file_name().unwrap();
        if path.is_file() {
            let destination_path = PathBuf::from(&target_asset_dir).join(file_name);
            fs::copy(path, destination_path)?;
        }
    }
    Ok(())
}
