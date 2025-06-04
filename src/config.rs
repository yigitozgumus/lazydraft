use std::fs;
use std::{env, fs::File, io::BufReader};
use toml;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub type ConfigResult<T> = Result<T, String>;

// Helper function to expand tilde in paths
fn expand_tilde(path: &str) -> String {
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_used: Option<String>,
    #[serde(flatten)]
    pub config: Config,
}

#[derive(Serialize, Deserialize)]
pub struct ActiveProject {
    pub name: String,
}

pub struct ProjectManager {
    config_dir: PathBuf,
    projects_dir: PathBuf,
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
    // Helper methods to get expanded paths
    pub fn get_source_dir(&self) -> Option<String> {
        self.source_dir.as_ref().map(|s| expand_tilde(s))
    }
    
    pub fn get_source_asset_dir(&self) -> Option<String> {
        self.source_asset_dir.as_ref().map(|s| expand_tilde(s))
    }
    
    pub fn get_target_dir(&self) -> Option<String> {
        self.target_dir.as_ref().map(|s| expand_tilde(s))
    }
    
    pub fn get_target_asset_dir(&self) -> Option<String> {
        self.target_asset_dir.as_ref().map(|s| expand_tilde(s))
    }

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
            return Some("target_asset_prefix".to_string());
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

impl ProjectConfig {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            name,
            description,
            created_at: Some(now.clone()),
            last_used: Some(now),
            config: Config {
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
            },
        }
    }

    pub fn update_last_used(&mut self) {
        self.last_used = Some(chrono::Utc::now().to_rfc3339());
    }
}

impl ProjectManager {
    pub fn new() -> ConfigResult<Self> {
        let home = env::var("HOME").map_err(|_| "HOME environment variable not set")?;
        let config_dir = PathBuf::from(format!("{}/.config/lazydraft", home));
        let projects_dir = config_dir.join("projects");

        // Ensure directories exist
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
        fs::create_dir_all(&projects_dir)
            .map_err(|e| format!("Failed to create projects directory: {}", e))?;

        Ok(Self {
            config_dir,
            projects_dir,
        })
    }

    pub fn list_projects(&self) -> ConfigResult<Vec<ProjectConfig>> {
        let mut projects = Vec::new();
        
        if !self.projects_dir.exists() {
            return Ok(projects);
        }

        let entries = fs::read_dir(&self.projects_dir)
            .map_err(|e| format!("Failed to read projects directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                match self.load_project_from_path(&path) {
                    Ok(project) => projects.push(project),
                    Err(e) => eprintln!("Warning: Failed to load project from {:?}: {}", path, e),
                }
            }
        }

        // Sort by last_used (most recent first)
        projects.sort_by(|a, b| {
            let a_time = a.last_used.as_deref().unwrap_or("");
            let b_time = b.last_used.as_deref().unwrap_or("");
            b_time.cmp(a_time)
        });

        Ok(projects)
    }

    pub fn create_project(&self, name: &str, description: Option<String>) -> ConfigResult<ProjectConfig> {
        let project_path = self.projects_dir.join(format!("{}.toml", name));
        
        if project_path.exists() {
            return Err(format!("Project '{}' already exists", name));
        }

        let project = ProjectConfig::new(name.to_string(), description);
        self.save_project(&project)?;
        Ok(project)
    }

    pub fn load_project(&self, name: &str) -> ConfigResult<ProjectConfig> {
        let project_path = self.projects_dir.join(format!("{}.toml", name));
        self.load_project_from_path(&project_path)
    }

    fn load_project_from_path(&self, path: &Path) -> ConfigResult<ProjectConfig> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read project file: {}", e))?;
        
        toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse project config: {}", e))
    }

    pub fn save_project(&self, project: &ProjectConfig) -> ConfigResult<()> {
        let project_path = self.projects_dir.join(format!("{}.toml", project.name));
        let contents = toml::to_string_pretty(project)
            .map_err(|e| format!("Failed to serialize project config: {}", e))?;
        
        fs::write(&project_path, contents)
            .map_err(|e| format!("Failed to write project file: {}", e))?;
        
        Ok(())
    }

    pub fn delete_project(&self, name: &str) -> ConfigResult<()> {
        let project_path = self.projects_dir.join(format!("{}.toml", name));
        
        if !project_path.exists() {
            return Err(format!("Project '{}' does not exist", name));
        }

        fs::remove_file(&project_path)
            .map_err(|e| format!("Failed to delete project file: {}", e))?;
        
        Ok(())
    }

    pub fn get_active_project(&self) -> ConfigResult<Option<String>> {
        let active_path = self.config_dir.join("active_project.toml");
        
        if !active_path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&active_path)
            .map_err(|e| format!("Failed to read active project file: {}", e))?;
        
        let active: ActiveProject = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse active project: {}", e))?;
        
        Ok(Some(active.name))
    }

    pub fn set_active_project(&self, name: &str) -> ConfigResult<()> {
        // Verify project exists
        self.load_project(name)?;
        
        let active = ActiveProject {
            name: name.to_string(),
        };
        
        let active_path = self.config_dir.join("active_project.toml");
        let contents = toml::to_string_pretty(&active)
            .map_err(|e| format!("Failed to serialize active project: {}", e))?;
        
        fs::write(&active_path, contents)
            .map_err(|e| format!("Failed to write active project file: {}", e))?;
        
        Ok(())
    }

    pub fn migrate_legacy_config(&self) -> ConfigResult<Option<String>> {
        let legacy_toml_path = self.config_dir.join("lazydraft.toml");
        let legacy_json_path = self.config_dir.join("lazydraft.json");
        
        // Check if we have a legacy config to migrate
        let legacy_config = if legacy_toml_path.exists() {
            let contents = fs::read_to_string(&legacy_toml_path)
                .map_err(|e| format!("Failed to read legacy TOML config: {}", e))?;
            Some(toml::from_str::<Config>(&contents)
                .map_err(|e| format!("Failed to parse legacy TOML config: {}", e))?)
        } else if legacy_json_path.exists() {
            let file = File::open(&legacy_json_path)
                .map_err(|e| format!("Failed to open legacy JSON config: {}", e))?;
            let reader = BufReader::new(file);
            Some(serde_json::from_reader(reader)
                .map_err(|e| format!("Failed to parse legacy JSON config: {}", e))?)
        } else {
            None
        };

        if let Some(config) = legacy_config {
            let project_name = "default".to_string();
            let mut project = ProjectConfig::new(
                project_name.clone(),
                Some("Migrated from legacy configuration".to_string())
            );
            project.config = config;
            
            self.save_project(&project)?;
            self.set_active_project(&project_name)?;
            
            // Clean up legacy files
            if legacy_toml_path.exists() {
                let _ = fs::remove_file(&legacy_toml_path);
            }
            if legacy_json_path.exists() {
                let _ = fs::remove_file(&legacy_json_path);
            }
            
            println!("Migrated legacy configuration to project '{}'", project_name);
            return Ok(Some(project_name));
        }
        
        Ok(None)
    }
}

pub fn validate_config() -> ConfigResult<Config> {
    let project_manager = ProjectManager::new()?;
    
    // First, try to migrate any legacy config
    project_manager.migrate_legacy_config()?;
    
    // Get active project or prompt for selection
    let active_project_name = match project_manager.get_active_project()? {
        Some(name) => name,
        None => {
            let projects = project_manager.list_projects()?;
            if projects.is_empty() {
                return Err("No projects found. Create a project with 'lazydraft project create <name>'".to_string());
            } else if projects.len() == 1 {
                let project_name = projects[0].name.clone();
                project_manager.set_active_project(&project_name)?;
                project_name
            } else {
                return Err("Multiple projects found but no active project set. Use 'lazydraft project switch <name>' to select a project".to_string());
            }
        }
    };
    
    // Load and update the active project
    let mut project = project_manager.load_project(&active_project_name)?;
    project.update_last_used();
    project_manager.save_project(&project)?;
    
    Ok(project.config)
}

pub fn get_project_manager() -> ConfigResult<ProjectManager> {
    ProjectManager::new()
}
