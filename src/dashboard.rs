use std::io;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::path::Path;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::ListState,
    Terminal,
};
use notify::{RecursiveMode, RecommendedWatcher, Watcher, Event as NotifyEvent, EventKind};

use crate::project::{get_project_manager, ProjectConfig, ProjectManager};
use crate::writing::{create_writing_list, Writing, update_writing_content_and_transfer};
use crate::asset::{get_asset_list_of_writing, transfer_asset_files};
use crate::views;

#[derive(Clone, PartialEq)]
pub enum ViewMode {
    Projects,
    Writings,
}

#[derive(Clone, PartialEq)]
pub enum PopupType {
    None,
    StageConfirm,
    RevertConfirm,
    OperationResult { success: bool, message: String },
}

pub struct Dashboard {
    pub project_manager: ProjectManager,
    pub projects: Vec<ProjectConfig>,
    pub active_project: Option<String>,
    pub selected_index: usize,
    pub show_help: bool,
    pub last_update: Instant,
    pub project_stats: Vec<ProjectStats>,
    pub view_mode: ViewMode,
    pub writings: Vec<Writing>,
    pub selected_writings_index: usize,
    pub popup_type: PopupType,
    pub staged_writings: Vec<String>,
    pub auto_stage_enabled: bool,
    pub popup_timestamp: Option<Instant>,
    pub last_message: Option<(String, bool, Instant)>,
    pub quit_requested: Option<Instant>,

    // File watching — private
    file_watcher: Option<RecommendedWatcher>,
    file_events_rx: Option<mpsc::Receiver<notify::Result<NotifyEvent>>>,
}

#[derive(Clone)]
pub struct ProjectStats {
    pub name: String,
    pub draft_count: usize,
    pub total_files: usize,
    pub last_activity: String,
    pub is_active: bool,
}

impl Dashboard {
    pub fn new() -> Result<Self, String> {
        let project_manager = get_project_manager()?;
        let projects = project_manager.list_projects()?;
        let active_project = project_manager.get_active_project()?;

        let mut dashboard = Self {
            project_manager,
            projects: projects.clone(),
            active_project,
            selected_index: 0,
            show_help: false,
            last_update: Instant::now(),
            project_stats: Vec::new(),
            view_mode: ViewMode::Projects,
            writings: Vec::new(),
            selected_writings_index: 0,
            popup_type: PopupType::None,
            staged_writings: Vec::new(),
            file_watcher: None,
            file_events_rx: None,
            auto_stage_enabled: true,
            popup_timestamp: None,
            last_message: None,
            quit_requested: None,
        };

        dashboard.update_project_stats()?;
        Ok(dashboard)
    }

    pub fn update_project_stats(&mut self) -> Result<(), String> {
        self.project_stats.clear();

        for project in &self.projects {
            let is_active = self.active_project.as_ref() == Some(&project.name);
            let (draft_count, total_files) = if project.config.get_source_dir().is_some() {
                match create_writing_list(&project.config) {
                    Ok(writings) => {
                        let drafts = writings.iter().filter(|w| w.is_draft).count();
                        (drafts, writings.len())
                    }
                    Err(_) => (0, 0),
                }
            } else {
                (0, 0)
            };

            let last_activity = project.last_used
                .as_ref()
                .map(|t| views::format_relative_time(t))
                .unwrap_or_else(|| "Never".to_string());

            self.project_stats.push(ProjectStats {
                name: project.name.clone(),
                draft_count,
                total_files,
                last_activity,
                is_active,
            });
        }
        Ok(())
    }

    pub fn next_project(&mut self) {
        if !self.projects.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.projects.len();
        }
    }

    pub fn previous_project(&mut self) {
        if !self.projects.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.projects.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn switch_to_selected_project(&mut self) -> Result<(), String> {
        if let Some(project) = self.projects.get(self.selected_index) {
            self.project_manager.set_active_project(&project.name)?;
            self.active_project = Some(project.name.clone());
            self.update_project_stats()?;
        }
        Ok(())
    }

    pub fn refresh_data(&mut self) -> Result<(), String> {
        self.projects = self.project_manager.list_projects()?;
        self.active_project = self.project_manager.get_active_project()?;
        self.update_project_stats()?;
        self.selected_index = self.selected_index.min(self.projects.len().saturating_sub(1));
        self.last_update = Instant::now();
        Ok(())
    }

    fn load_writings_for_selected_project(&mut self) -> Result<(), String> {
        if let Some(project) = self.projects.get(self.selected_index) {
            if project.config.get_source_dir().is_some() {
                match create_writing_list(&project.config) {
                    Ok(writings) => {
                        self.writings = writings;
                        self.selected_writings_index = 0;
                    }
                    Err(e) => return Err(format!("Failed to load writings: {}", e)),
                }
            } else {
                self.writings.clear();
                return Err("Project not configured".to_string());
            }
        }
        Ok(())
    }

    pub fn next_writing(&mut self) {
        if !self.writings.is_empty() {
            self.selected_writings_index = (self.selected_writings_index + 1) % self.writings.len();
        }
    }

    pub fn previous_writing(&mut self) {
        if !self.writings.is_empty() {
            self.selected_writings_index = if self.selected_writings_index == 0 {
                self.writings.len() - 1
            } else {
                self.selected_writings_index - 1
            };
        }
    }

    pub fn switch_to_writings_view(&mut self) -> Result<(), String> {
        self.load_writings_for_selected_project()?;
        self.view_mode = ViewMode::Writings;
        Ok(())
    }

    pub fn switch_to_projects_view(&mut self) {
        self.view_mode = ViewMode::Projects;
    }

    pub fn show_stage_confirm_popup(&mut self) {
        if self.view_mode == ViewMode::Writings && !self.writings.is_empty() {
            if let Some(writing) = self.writings.get(self.selected_writings_index) {
                if writing.is_draft && !self.staged_writings.contains(&writing.path) {
                    self.show_popup(PopupType::StageConfirm);
                }
            }
        }
    }

    pub fn show_revert_confirm_popup(&mut self) {
        if self.view_mode == ViewMode::Writings && !self.writings.is_empty() {
            if let Some(writing) = self.writings.get(self.selected_writings_index) {
                if self.staged_writings.contains(&writing.path) {
                    self.show_popup(PopupType::RevertConfirm);
                }
            }
        }
    }

    pub fn stage_selected_writing(&mut self) -> Result<(), String> {
        if let Some(writing) = self.writings.get(self.selected_writings_index).cloned() {
            if let Some(project) = self.projects.get(self.selected_index) {
                let asset_list = get_asset_list_of_writing(&writing, &project.config)
                    .map_err(|e| format!("Failed to create asset list: {}", e))?;
                let asset_count = asset_list.len();

                match update_writing_content_and_transfer(&project.config, &writing, &asset_list) {
                    Ok(_) => {
                        let asset_result = if asset_count > 0 {
                            match transfer_asset_files(&project.config, &asset_list) {
                                Ok(_) => format!(" ({} assets transferred)", asset_count),
                                Err(e) => format!(" (Warning: Asset transfer failed: {})", e),
                            }
                        } else {
                            " (no assets found)".to_string()
                        };

                        let writing_path = writing.path.clone();
                        let writing_title = writing.title.clone();
                        self.staged_writings.push(writing_path.clone());
                        let _ = self.add_file_to_watch(&writing_path);
                        let _ = self.load_writings_for_selected_project();

                        self.show_popup(PopupType::OperationResult {
                            success: true,
                            message: format!("Successfully staged: {}{}", writing_title, asset_result),
                        });
                        Ok(())
                    }
                    Err(e) => {
                        self.show_popup(PopupType::OperationResult {
                            success: false,
                            message: format!("Failed to stage: {}", e),
                        });
                        Err(format!("Staging failed: {}", e))
                    }
                }
            } else {
                Err("No project selected".to_string())
            }
        } else {
            Err("No writing selected".to_string())
        }
    }

    pub fn revert_selected_writing(&mut self) -> Result<(), String> {
        if let Some(writing) = self.writings.get(self.selected_writings_index).cloned() {
            if let Some(pos) = self.staged_writings.iter().position(|x| x == &writing.path) {
                let writing_path = writing.path.clone();
                let writing_title = writing.title.clone();
                self.staged_writings.remove(pos);
                let _ = self.remove_file_from_watch(&writing_path);

                self.show_popup(PopupType::OperationResult {
                    success: true,
                    message: format!("Reverted staging for: {}", writing_title),
                });
                Ok(())
            } else {
                Err("Writing is not staged".to_string())
            }
        } else {
            Err("No writing selected".to_string())
        }
    }

    pub fn close_popup(&mut self) {
        self.popup_type = PopupType::None;
        self.popup_timestamp = None;
    }

    pub fn show_popup(&mut self, popup: PopupType) {
        if let PopupType::OperationResult { ref message, success } = popup {
            self.last_message = Some((message.clone(), success, Instant::now()));
        }
        self.popup_type = popup;
        self.popup_timestamp = Some(Instant::now());
        self.quit_requested = None;
    }

    pub fn request_quit(&mut self) {
        // show_popup clears quit_requested, so we set it after
        self.show_popup(PopupType::OperationResult {
            success: true,
            message: "Press q again to quit, or Esc to cancel.".to_string(),
        });
        self.quit_requested = Some(Instant::now());
    }

    pub fn toggle_auto_stage(&mut self) {
        self.auto_stage_enabled = !self.auto_stage_enabled;
        let status = if self.auto_stage_enabled { "enabled" } else { "disabled" };
        self.show_popup(PopupType::OperationResult {
            success: true,
            message: format!("Auto-staging {}", status),
        });
    }

    // ── File watching (private) ──────────────────────────────────────────

    fn start_file_watching(&mut self) -> Result<(), String> {
        if self.file_watcher.is_some() {
            return Ok(());
        }
        let (tx, rx) = mpsc::channel();

        match RecommendedWatcher::new(tx, notify::Config::default()) {
            Ok(mut watcher) => {
                for writing_path in &self.staged_writings {
                    let _ = watcher.watch(Path::new(writing_path), RecursiveMode::NonRecursive);
                }
                self.file_watcher = Some(watcher);
                self.file_events_rx = Some(rx);
                Ok(())
            }
            Err(e) => Err(format!("Failed to create file watcher: {}", e)),
        }
    }

    fn add_file_to_watch(&mut self, file_path: &str) -> Result<(), String> {
        if let Some(ref mut watcher) = self.file_watcher {
            watcher.watch(Path::new(file_path), RecursiveMode::NonRecursive)
                .map_err(|e| format!("Failed to watch file {}: {}", file_path, e))?;
        } else {
            self.start_file_watching()?;
            if let Some(ref mut watcher) = self.file_watcher {
                watcher.watch(Path::new(file_path), RecursiveMode::NonRecursive)
                    .map_err(|e| format!("Failed to watch file {}: {}", file_path, e))?;
            }
        }
        Ok(())
    }

    fn remove_file_from_watch(&mut self, file_path: &str) -> Result<(), String> {
        if let Some(ref mut watcher) = self.file_watcher {
            watcher.unwatch(Path::new(file_path))
                .map_err(|e| format!("Failed to unwatch file {}: {}", file_path, e))?;
        }
        Ok(())
    }

    pub fn process_file_events(&mut self) {
        let mut paths_to_restage = Vec::new();

        if let Some(ref rx) = self.file_events_rx {
            while let Ok(event_result) = rx.try_recv() {
                match event_result {
                    Ok(event) => {
                        if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                            if self.auto_stage_enabled {
                                for path in event.paths {
                                    let path_str = path.to_string_lossy().to_string();
                                    if self.staged_writings.contains(&path_str) {
                                        paths_to_restage.push(path_str);
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        for path_str in paths_to_restage {
            self.auto_restage_writing(&path_str);
        }
    }

    fn auto_restage_writing(&mut self, file_path: &str) {
        if let Some(writing) = self.writings.iter().find(|w| w.path == file_path).cloned() {
            if let Some(project) = self.projects.get(self.selected_index) {
                match get_asset_list_of_writing(&writing, &project.config) {
                    Ok(asset_list) => {
                        match update_writing_content_and_transfer(&project.config, &writing, &asset_list) {
                            Ok(_) => {
                                let _ = transfer_asset_files(&project.config, &asset_list);
                                let _ = self.load_writings_for_selected_project();
                                self.show_popup(PopupType::OperationResult {
                                    success: true,
                                    message: format!("Auto-staged: {}", writing.title),
                                });
                            }
                            Err(_) => {}
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    }
}

// ── Dashboard entry point ───────────────────────────────────────────────────

pub fn run_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut dashboard = Dashboard::new().map_err(|e| format!("Failed to create dashboard: {}", e))?;
    let res = run_app(&mut terminal, &mut dashboard);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    dashboard: &mut Dashboard,
) -> io::Result<()> {
    let mut projects_list_state = ListState::default();
    let mut writings_list_state = ListState::default();

    loop {
        if !dashboard.projects.is_empty() {
            let idx = dashboard.selected_index.min(dashboard.projects.len() - 1);
            projects_list_state.select(Some(idx));
        }
        if !dashboard.writings.is_empty() {
            let idx = dashboard.selected_writings_index.min(dashboard.writings.len() - 1);
            writings_list_state.select(Some(idx));
        }

        dashboard.process_file_events();

        terminal.draw(|f| views::ui(f, dashboard, &mut projects_list_state, &mut writings_list_state))?;

        if let Some(timestamp) = dashboard.popup_timestamp {
            if timestamp.elapsed() >= Duration::from_secs(2) {
                dashboard.close_popup();
            }
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match &dashboard.popup_type {
                        PopupType::StageConfirm => {
                            match key.code {
                                KeyCode::Char('y') | KeyCode::Enter => { let _ = dashboard.stage_selected_writing(); }
                                KeyCode::Char('n') | KeyCode::Esc => { dashboard.close_popup(); }
                                _ => {}
                            }
                            continue;
                        }
                        PopupType::RevertConfirm => {
                            match key.code {
                                KeyCode::Char('y') | KeyCode::Enter => { let _ = dashboard.revert_selected_writing(); }
                                KeyCode::Char('n') | KeyCode::Esc => { dashboard.close_popup(); }
                                _ => {}
                            }
                            continue;
                        }
                        PopupType::OperationResult { .. } => {
                            if key.code == KeyCode::Char('q') && dashboard.quit_requested.is_some() {
                                return Ok(());
                            }
                            match key.code {
                                KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
                                    dashboard.quit_requested = None;
                                    dashboard.close_popup();
                                }
                                _ => { dashboard.quit_requested = None; }
                            }
                            continue;
                        }
                        PopupType::None => {}
                    }

                    match key.code {
                        KeyCode::Char('q') => {
                            if dashboard.quit_requested.is_some() {
                                return Ok(());
                            }
                            dashboard.request_quit();
                        }
                        KeyCode::Esc => {
                            if dashboard.quit_requested.is_some() {
                                dashboard.quit_requested = None;
                                dashboard.close_popup();
                            } else {
                                return Ok(());
                            }
                        }
                        KeyCode::Char('h') | KeyCode::F(1) => { dashboard.show_help = !dashboard.show_help; }
                        KeyCode::Char('r') | KeyCode::F(5) => { let _ = dashboard.refresh_data(); }
                        KeyCode::Char('s') => { dashboard.show_stage_confirm_popup(); }
                        KeyCode::Char('u') => { dashboard.show_revert_confirm_popup(); }
                        KeyCode::Char('a') => { dashboard.toggle_auto_stage(); }
                        KeyCode::Down | KeyCode::Char('j') => {
                            match dashboard.view_mode {
                                ViewMode::Projects => dashboard.next_project(),
                                ViewMode::Writings => dashboard.next_writing(),
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            match dashboard.view_mode {
                                ViewMode::Projects => dashboard.previous_project(),
                                ViewMode::Writings => dashboard.previous_writing(),
                            }
                        }
                        KeyCode::Right => {
                            if dashboard.view_mode == ViewMode::Projects {
                                let _ = dashboard.switch_to_writings_view();
                            }
                        }
                        KeyCode::Left => {
                            if dashboard.view_mode == ViewMode::Writings {
                                dashboard.switch_to_projects_view();
                            }
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if dashboard.view_mode == ViewMode::Projects {
                                let _ = dashboard.switch_to_selected_project();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if dashboard.last_update.elapsed() > Duration::from_secs(30) {
            let _ = dashboard.refresh_data();
        }
    }
}
