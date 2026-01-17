use std::io;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::thread;
use std::path::Path;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Event as NotifyEvent, EventKind};

use crate::config::{get_project_manager, ProjectConfig, ProjectManager};
use crate::tui::Theme;
use crate::writing::{create_writing_list, Writing, update_writing_content_and_transfer};
use crate::asset::{get_asset_list_of_writing, transfer_asset_files};

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
    project_manager: ProjectManager,
    projects: Vec<ProjectConfig>,
    active_project: Option<String>,
    selected_index: usize,
    list_state: ListState,
    show_help: bool,
    last_update: Instant,
    project_stats: Vec<ProjectStats>,
    view_mode: ViewMode,
    writings: Vec<Writing>,
    writings_list_state: ListState,
    selected_writings_index: usize,
    popup_type: PopupType,
    staged_writings: Vec<String>, // Track staged writing paths
    file_watcher: Option<RecommendedWatcher>,
    file_events_rx: Option<mpsc::Receiver<notify::Result<NotifyEvent>>>,
    auto_stage_enabled: bool,
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
            list_state: ListState::default(),
            show_help: false,
            last_update: Instant::now(),
            project_stats: Vec::new(),
            view_mode: ViewMode::Projects,
            writings: Vec::new(),
            writings_list_state: ListState::default(),
            selected_writings_index: 0,
            popup_type: PopupType::None,
            staged_writings: Vec::new(),
            file_watcher: None,
            file_events_rx: None,
            auto_stage_enabled: true,
        };
        
        dashboard.update_project_stats()?;
        dashboard.update_selection();
        
        Ok(dashboard)
    }
    
    fn update_project_stats(&mut self) -> Result<(), String> {
        self.project_stats.clear();
        
        for project in &self.projects {
            let is_active = self.active_project.as_ref() == Some(&project.name);
            
            // Count drafts and total files - handle incomplete configs gracefully
            let (draft_count, total_files) = if project.config.get_source_dir().is_some() {
                match create_writing_list(&project.config) {
                    Ok(writings) => {
                        let drafts = writings.iter().filter(|w| w.is_draft).count();
                        (drafts, writings.len())
                    }
                    Err(_) => (0, 0),
                }
            } else {
                // Project not configured yet
                (0, 0)
            };
            
            let last_activity = project.last_used
                .as_ref()
                .map(|t| format_relative_time(t))
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
    
    fn update_selection(&mut self) {
        if !self.projects.is_empty() {
            self.selected_index = self.selected_index.min(self.projects.len() - 1);
            self.list_state.select(Some(self.selected_index));
        }
    }
    
    fn next_project(&mut self) {
        if !self.projects.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.projects.len();
            self.update_selection();
        }
    }
    
    fn previous_project(&mut self) {
        if !self.projects.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.projects.len() - 1
            } else {
                self.selected_index - 1
            };
            self.update_selection();
        }
    }
    
    fn switch_to_selected_project(&mut self) -> Result<(), String> {
        if let Some(project) = self.projects.get(self.selected_index) {
            self.project_manager.set_active_project(&project.name)?;
            self.active_project = Some(project.name.clone());
            self.update_project_stats()?;
        }
        Ok(())
    }
    
    fn refresh_data(&mut self) -> Result<(), String> {
        self.projects = self.project_manager.list_projects()?;
        self.active_project = self.project_manager.get_active_project()?;
        self.update_project_stats()?;
        self.update_selection();
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
                        self.update_writings_selection();
                    }
                    Err(e) => {
                        return Err(format!("Failed to load writings: {}", e));
                    }
                }
            } else {
                self.writings.clear();
                return Err("Project not configured".to_string());
            }
        }
        Ok(())
    }
    
    fn update_writings_selection(&mut self) {
        if !self.writings.is_empty() {
            self.selected_writings_index = self.selected_writings_index.min(self.writings.len() - 1);
            self.writings_list_state.select(Some(self.selected_writings_index));
        }
    }
    
    fn next_writing(&mut self) {
        if !self.writings.is_empty() {
            self.selected_writings_index = (self.selected_writings_index + 1) % self.writings.len();
            self.update_writings_selection();
        }
    }
    
    fn previous_writing(&mut self) {
        if !self.writings.is_empty() {
            self.selected_writings_index = if self.selected_writings_index == 0 {
                self.writings.len() - 1
            } else {
                self.selected_writings_index - 1
            };
            self.update_writings_selection();
        }
    }
    
    fn switch_to_writings_view(&mut self) -> Result<(), String> {
        self.load_writings_for_selected_project()?;
        self.view_mode = ViewMode::Writings;
        Ok(())
    }
    
    fn switch_to_projects_view(&mut self) {
        self.view_mode = ViewMode::Projects;
    }
    
    fn show_stage_confirm_popup(&mut self) {
        if self.view_mode == ViewMode::Writings && !self.writings.is_empty() {
            if let Some(writing) = self.writings.get(self.selected_writings_index) {
                if writing.is_draft && !self.staged_writings.contains(&writing.path) {
                    self.popup_type = PopupType::StageConfirm;
                }
            }
        }
    }
    
    fn show_revert_confirm_popup(&mut self) {
        if self.view_mode == ViewMode::Writings && !self.writings.is_empty() {
            if let Some(writing) = self.writings.get(self.selected_writings_index) {
                if self.staged_writings.contains(&writing.path) {
                    self.popup_type = PopupType::RevertConfirm;
                }
            }
        }
    }
    
    fn stage_selected_writing(&mut self) -> Result<(), String> {
        if let Some(writing) = self.writings.get(self.selected_writings_index).cloned() {
            if let Some(project) = self.projects.get(self.selected_index) {
                // Create asset list for the staging process
                let asset_list = get_asset_list_of_writing(&writing, &project.config)
                    .map_err(|e| format!("Failed to create asset list: {}", e))?;
                
                let asset_count = asset_list.len();
                
                // Stage the writing
                match update_writing_content_and_transfer(&project.config, &writing, &asset_list) {
                    Ok(_) => {
                        // Also transfer assets
                        let asset_result = if asset_count > 0 {
                            match transfer_asset_files(&project.config, &asset_list) {
                                Ok(_) => format!(" ({} assets transferred)", asset_count),
                                Err(e) => format!(" (Warning: Asset transfer failed: {})", e),
                            }
                        } else {
                            " (no assets found)".to_string()
                        };
                        
                        // Add to staged writings and start watching
                        let writing_path = writing.path.clone();
                        let writing_title = writing.title.clone();
                        self.staged_writings.push(writing_path.clone());
                        if let Err(_) = self.add_file_to_watch(&writing_path) {
                            // File watching failure is not critical, just continue
                        }
                        
                        // Refresh the writings list to reflect any changes (like draft status)
                        if let Err(_) = self.load_writings_for_selected_project() {
                            // Refresh failure is not critical, just continue
                        }
                        
                        self.popup_type = PopupType::OperationResult {
                            success: true,
                            message: format!("Successfully staged: {}{}", writing_title, asset_result),
                        };
                        Ok(())
                    }
                    Err(e) => {
                        self.popup_type = PopupType::OperationResult {
                            success: false,
                            message: format!("Failed to stage: {}", e),
                        };
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
    
    fn revert_selected_writing(&mut self) -> Result<(), String> {
        if let Some(writing) = self.writings.get(self.selected_writings_index).cloned() {
            if let Some(pos) = self.staged_writings.iter().position(|x| x == &writing.path) {
                // Remove from staged writings and stop watching
                let writing_path = writing.path.clone();
                let writing_title = writing.title.clone();
                self.staged_writings.remove(pos);
                if let Err(e) = self.remove_file_from_watch(&writing_path) {
                    // Silently continue if we can't stop watching a specific file
                }
                
                self.popup_type = PopupType::OperationResult {
                    success: true,
                    message: format!("Reverted staging for: {} (auto-staging disabled)", writing_title),
                };
                Ok(())
            } else {
                Err("Writing is not staged".to_string())
            }
        } else {
            Err("No writing selected".to_string())
        }
    }
    
    fn close_popup(&mut self) {
        self.popup_type = PopupType::None;
    }
    
    fn start_file_watching(&mut self) -> Result<(), String> {
        if self.file_watcher.is_some() {
            return Ok(()); // Already watching
        }
        
        let (tx, rx) = mpsc::channel();
        
        match RecommendedWatcher::new(tx, notify::Config::default()) {
            Ok(mut watcher) => {
                // Watch all staged writing files
                for writing_path in &self.staged_writings {
                    if let Err(_) = watcher.watch(Path::new(writing_path), RecursiveMode::NonRecursive) {
                        // Silently continue if we can't watch a specific file
                    }
                }
                
                self.file_watcher = Some(watcher);
                self.file_events_rx = Some(rx);
                Ok(())
            }
            Err(e) => Err(format!("Failed to create file watcher: {}", e))
        }
    }
    
    fn add_file_to_watch(&mut self, file_path: &str) -> Result<(), String> {
        if let Some(ref mut watcher) = self.file_watcher {
            watcher.watch(Path::new(file_path), RecursiveMode::NonRecursive)
                .map_err(|e| format!("Failed to watch file {}: {}", file_path, e))?;
        } else {
            // Start watching if not already started
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
    
    fn process_file_events(&mut self) {
        let mut paths_to_restage = Vec::new();
        
        if let Some(ref rx) = self.file_events_rx {
            while let Ok(event_result) = rx.try_recv() {
                match event_result {
                    Ok(event) => {
                        match event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) => {
                                if self.auto_stage_enabled {
                                    for path in event.paths {
                                        let path_str = path.to_string_lossy().to_string();
                                        if self.staged_writings.contains(&path_str) {
                                            paths_to_restage.push(path_str);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {
                        // Silently ignore file watch errors to avoid breaking TUI
                    }
                }
            }
        }
        
        // Process the collected paths
        for path_str in paths_to_restage {
            self.auto_restage_writing(&path_str);
        }
    }
    
    fn auto_restage_writing(&mut self, file_path: &str) {
        // Find the writing that matches this file path
        if let Some(writing) = self.writings.iter().find(|w| w.path == file_path).cloned() {
            if let Some(project) = self.projects.get(self.selected_index) {
                // Auto-restage the writing
                match get_asset_list_of_writing(&writing, &project.config) {
                    Ok(asset_list) => {
                        match update_writing_content_and_transfer(&project.config, &writing, &asset_list) {
                            Ok(_) => {
                                // Also transfer assets
                                if let Err(_) = transfer_asset_files(&project.config, &asset_list) {
                                    // Silently continue on asset transfer failure
                                }
                                
                                // Refresh the writings list to reflect any changes
                                if let Err(_) = self.load_writings_for_selected_project() {
                                    // Silently continue on refresh failure
                                }
                                
                                // Show brief success notification
                                self.popup_type = PopupType::OperationResult {
                                    success: true,
                                    message: format!("Auto-staged: {}", writing.title),
                                };
                                // Auto-close the popup after a short time
                                thread::spawn(move || {
                                    thread::sleep(Duration::from_secs(2));
                                });
                            }
                            Err(_) => {
                                // Silently continue on staging failure
                            }
                        }
                    }
                    Err(_) => {
                        // Silently continue on asset list failure
                    }
                }
            }
        }
    }
    
    fn toggle_auto_stage(&mut self) {
        self.auto_stage_enabled = !self.auto_stage_enabled;
        let status = if self.auto_stage_enabled { "enabled" } else { "disabled" };
        self.popup_type = PopupType::OperationResult {
            success: true,
            message: format!("Auto-staging {}", status),
        };
    }
}

pub fn run_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create dashboard
    let mut dashboard = Dashboard::new().map_err(|e| format!("Failed to create dashboard: {}", e))?;
    
    let res = run_app(&mut terminal, &mut dashboard);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
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
    loop {
        // Process file events for auto-staging
        dashboard.process_file_events();
        
        terminal.draw(|f| ui(f, dashboard))?;
        
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Handle popup interactions first
                    match &dashboard.popup_type {
                        PopupType::StageConfirm => {
                            match key.code {
                                KeyCode::Char('y') | KeyCode::Enter => {
                                    if let Err(_) = dashboard.stage_selected_writing() {
                                        // Error is already shown in popup, no need for eprintln
                                    }
                                }
                                KeyCode::Char('n') | KeyCode::Esc => {
                                    dashboard.close_popup();
                                }
                                _ => {}
                            }
                            continue;
                        }
                        PopupType::RevertConfirm => {
                            match key.code {
                                KeyCode::Char('y') | KeyCode::Enter => {
                                    if let Err(_) = dashboard.revert_selected_writing() {
                                        // Error is already shown in popup, no need for eprintln
                                    }
                                }
                                KeyCode::Char('n') | KeyCode::Esc => {
                                    dashboard.close_popup();
                                }
                                _ => {}
                            }
                            continue;
                        }
                        PopupType::OperationResult { .. } => {
                            match key.code {
                                KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
                                    dashboard.close_popup();
                                }
                                _ => {}
                            }
                            continue;
                        }
                        PopupType::None => {
                            // Handle normal navigation
                        }
                    }
                    
                    // Normal event handling when no popup is active
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('h') | KeyCode::F(1) => {
                            dashboard.show_help = !dashboard.show_help;
                        }
                        KeyCode::Char('r') | KeyCode::F(5) => {
                            if let Err(_) = dashboard.refresh_data() {
                                // Silently continue on refresh error
                            }
                        }
                        KeyCode::Char('s') => {
                            // Stage operation
                            dashboard.show_stage_confirm_popup();
                        }
                        KeyCode::Char('u') => {
                            // Revert/undo staging operation (changed from 'r' to avoid conflict with refresh)
                            dashboard.show_revert_confirm_popup();
                        }
                        KeyCode::Char('a') => {
                            // Toggle auto-staging
                            dashboard.toggle_auto_stage();
                        }
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
                                if let Err(_) = dashboard.switch_to_writings_view() {
                                    // Silently continue on view switch error
                                }
                            }
                        }
                        KeyCode::Left => {
                            if dashboard.view_mode == ViewMode::Writings {
                                dashboard.switch_to_projects_view();
                            }
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if dashboard.view_mode == ViewMode::Projects {
                                if let Err(_) = dashboard.switch_to_selected_project() {
                                    // Silently continue on project switch error
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Auto-refresh every 30 seconds
        if dashboard.last_update.elapsed() > Duration::from_secs(30) {
            if let Err(_) = dashboard.refresh_data() {
                // Silently continue on auto-refresh error
            }
        }
    }
}

fn ui(f: &mut Frame, dashboard: &Dashboard) {
    let theme = Theme::default();
    if dashboard.show_help {
        draw_help_popup(f, &theme);
        return;
    }
    
    let size = f.size();
    
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(size);
    
    // Header
    let header_title = match dashboard.view_mode {
        ViewMode::Projects => "LazyDraft Dashboard - Projects",
        ViewMode::Writings => {
            if let Some(project) = dashboard.projects.get(dashboard.selected_index) {
                &format!("LazyDraft Dashboard - {} Writings", project.name)
            } else {
                "LazyDraft Dashboard - Writings"
            }
        }
    };
    
    let header = Paragraph::new(header_title)
        .style(theme.header_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
    f.render_widget(header, chunks[0]);
    
    // Main content based on view mode
    match dashboard.view_mode {
        ViewMode::Projects => draw_projects_view(f, dashboard, chunks[1], &theme),
        ViewMode::Writings => draw_writings_view(f, dashboard, chunks[1], &theme),
    }
    
    // Footer
    draw_footer(f, chunks[2], dashboard, &theme);
    
    // Render popup if active
    match &dashboard.popup_type {
        PopupType::StageConfirm => draw_stage_confirm_popup(f, &theme),
        PopupType::RevertConfirm => draw_revert_confirm_popup(f, &theme),
        PopupType::OperationResult { success, message } => {
            draw_operation_result_popup(f, *success, message, &theme)
        }
        PopupType::None => {}
    }
}

fn draw_projects_view(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    // Main content layout
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Projects list
            Constraint::Percentage(50),  // Project details & stats
        ])
        .split(area);
    
    // Projects list
    draw_projects_list(f, dashboard, main_chunks[0], theme);
    
    // Right panel layout
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Project details
            Constraint::Percentage(40),  // Overall stats
        ])
        .split(main_chunks[1]);
    
    // Project details
    draw_project_details(f, dashboard, right_chunks[0], theme);
    
    // Overall stats
    draw_overall_stats(f, dashboard, right_chunks[1], theme);
}

fn draw_projects_list(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    let items: Vec<ListItem> = if dashboard.project_stats.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No projects found",
            theme.muted_style(),
        )))]
    } else {
        dashboard
            .project_stats
            .iter()
            .enumerate()
            .map(|(i, stats)| {
                let project = &dashboard.projects[i];
                let is_configured =
                    project.config.get_source_dir().is_some() && project.config.get_target_dir().is_some();

                let indicator = if stats.is_active { "●" } else { " " };
                let config_indicator = if !is_configured { " ⚠" } else { "" };
                let draft_info = if stats.draft_count > 0 && is_configured {
                    format!(" ({} drafts)", stats.draft_count)
                } else {
                    String::new()
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} {}{}", indicator, stats.name, config_indicator),
                        if stats.is_active {
                            theme.success_style().add_modifier(Modifier::BOLD)
                        } else if !is_configured {
                            theme.danger_style()
                        } else {
                            Style::default().fg(theme.text)
                        },
                    ),
                    Span::styled(draft_info, theme.warning_style()),
                ]);

                ListItem::new(line)
            })
            .collect()
    };
    
    let projects_list = List::new(items)
        .block(
            Block::default()
                .title("Projects")
                .borders(Borders::ALL)
                .border_style(theme.border_style())
        )
        .highlight_style(theme.highlight_style())
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(projects_list, area, &mut dashboard.list_state.clone());
}

fn draw_writings_view(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    // Main content layout
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),  // Writings list
            Constraint::Percentage(40),  // Writing details
        ])
        .split(area);
    
    // Writings list
    draw_writings_list(f, dashboard, main_chunks[0], theme);
    
    // Writing details
    draw_writing_details(f, dashboard, main_chunks[1], theme);
}

fn draw_writings_list(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    let items: Vec<ListItem> = if dashboard.writings.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No writings found",
            theme.muted_style(),
        )))]
    } else {
        dashboard
            .writings
            .iter()
            .map(|writing| {
                let status_icon = if writing.is_draft { "📝" } else { "✅" };
                let status_color = if writing.is_draft { theme.warning } else { theme.success };

                // Check if writing is staged and show auto-staging status
                let staged_indicator = if dashboard.staged_writings.contains(&writing.path) {
                    if dashboard.auto_stage_enabled {
                        " 🚀⚡" // Staged with auto-staging
                    } else {
                        " 🚀" // Staged without auto-staging
                    }
                } else {
                    ""
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} {}", status_icon, writing.title),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(
                        staged_indicator,
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect()
    };
    
    let writings_count = dashboard.writings.len();
    let staged_count = dashboard.staged_writings.len();
    let auto_stage_status = if dashboard.auto_stage_enabled { "ON" } else { "OFF" };
    
    let title = if staged_count > 0 {
        format!("Writings ({} total, {} staged, auto-stage: {})", 
                writings_count, staged_count, auto_stage_status)
    } else {
        format!("Writings ({} total)", writings_count)
    };
    
    let writings_list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(theme.border_style())
        )
        .highlight_style(theme.highlight_style())
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(writings_list, area, &mut dashboard.writings_list_state.clone());
}

fn draw_writing_details(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    let content = if let Some(writing) = dashboard.writings.get(dashboard.selected_writings_index) {
        let status = if writing.is_draft { "Draft" } else { "Published" };
        let status_color = if writing.is_draft { theme.warning } else { theme.success };
        
        let lines = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&writing.title),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(status, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Path: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&writing.path),
            ]),
        ];
        
        Text::from(lines)
    } else {
        Text::from(Line::from(Span::styled(
            "No writing selected",
            theme.muted_style(),
        )))
    };
    
    let details = Paragraph::new(content)
        .block(
            Block::default()
                .title("Writing Details")
                .borders(Borders::ALL)
                .border_style(theme.border_style())
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(details, area);
}

fn draw_project_details(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    let content = if let Some(stats) = dashboard.project_stats.get(dashboard.selected_index) {
        let project = &dashboard.projects[dashboard.selected_index];
        
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&stats.name),
            ]),
        ];
        
        if let Some(desc) = &project.description {
            lines.push(Line::from(vec![
                Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(desc),
            ]));
        }
        
        lines.extend(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    if stats.is_active { "Active" } else { "Inactive" },
                    if stats.is_active {
                        theme.success_style()
                    } else {
                        theme.muted_style()
                    }
                ),
            ]),
        ]);
        
        // Check if project is configured
        let is_configured = project.config.get_source_dir().is_some() && project.config.get_target_dir().is_some();
        
        if is_configured {
            lines.extend(vec![
                Line::from(vec![
                    Span::styled("Drafts: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        stats.draft_count.to_string(),
                        if stats.draft_count > 0 {
                            theme.warning_style()
                        } else {
                            theme.success_style()
                        }
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Total Files: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(stats.total_files.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Last Activity: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&stats.last_activity),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Source: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(project.config.get_source_dir().unwrap_or_else(|| "not set".to_string())),
                ]),
                Line::from(vec![
                    Span::styled("Target: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(project.config.get_target_dir().unwrap_or_else(|| "not set".to_string())),
                ]),
            ]);
        } else {
            lines.extend(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "Configuration Required",
                        theme.danger_style().add_modifier(Modifier::BOLD)
                    ),
                ]),
                Line::from(""),
                Line::from("This project needs to be configured before use."),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Run: "),
                    Span::styled(
                        format!("lazydraft config --edit --project {}", stats.name),
                        Style::default().fg(theme.accent)
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Source: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("not set", theme.danger_style()),
                ]),
                Line::from(vec![
                    Span::styled("Target: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("not set", theme.danger_style()),
                ]),
            ]);
        }
        
        Text::from(lines)
    } else {
        Text::from(Line::from(Span::styled(
            "No project selected",
            theme.muted_style(),
        )))
    };
    
    let details = Paragraph::new(content)
        .block(
            Block::default()
                .title("Project Details")
                .borders(Borders::ALL)
                .border_style(theme.border_style())
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(details, area);
}

fn draw_overall_stats(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    let total_projects = dashboard.projects.len();
    let total_drafts: usize = dashboard.project_stats.iter().map(|s| s.draft_count).sum();
    let total_files: usize = dashboard.project_stats.iter().map(|s| s.total_files).sum();
    let active_projects = dashboard.project_stats.iter().filter(|s| s.is_active).count();
    
    let content = Text::from(vec![
        Line::from(vec![
            Span::styled("Total Projects: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(total_projects.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Active Projects: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                active_projects.to_string(),
                theme.success_style()
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Drafts: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                total_drafts.to_string(),
                if total_drafts > 0 {
                    theme.warning_style()
                } else {
                    theme.success_style()
                }
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Files: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(total_files.to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Last Update: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}s ago", dashboard.last_update.elapsed().as_secs()),
                theme.muted_style(),
            ),
        ]),
    ]);
    
    let stats = Paragraph::new(content)
        .block(
            Block::default()
                .title("Overall Statistics")
                .borders(Borders::ALL)
                .border_style(theme.border_style())
        );
    
    f.render_widget(stats, area);
}

fn draw_footer(f: &mut Frame, area: Rect, dashboard: &Dashboard, theme: &Theme) {
    let footer_text = match dashboard.view_mode {
        ViewMode::Projects => {
            "q: Quit | h: Help | r: Refresh | ↑↓: Navigate | →: View Writings | Enter: Switch Project"
                .to_string()
        }
        ViewMode::Writings => {
            let auto_stage = if dashboard.auto_stage_enabled { "ON" } else { "OFF" };
            format!(
                "q: Quit | h: Help | r: Refresh | ↑↓: Navigate | ←: Back | s: Stage | u: Revert | a: Auto-stage ({})",
                auto_stage
            )
        }
    };
    
    let footer = Paragraph::new(footer_text)
        .style(theme.muted_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
    f.render_widget(footer, area);
}

fn draw_help_popup(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(70, 60, f.size());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    let help_text = vec![
        Line::from("LazyDraft Dashboard Help"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  ↑/↓ or j/k    Navigate up/down"),
        Line::from("  ←/→           Switch between views"),
        Line::from("  Enter/Space   Switch to selected project"),
        Line::from(""),
        Line::from("Operations:"),
        Line::from("  s             Stage selected writing (Writings view)"),
        Line::from("  u             Revert/undo staging (Writings view)"),
        Line::from("  a             Toggle auto-staging (Writings view)"),
        Line::from("  r/F5          Refresh data"),
        Line::from(""),
        Line::from("Auto-staging:"),
        Line::from("  When enabled, staged writings are automatically"),
        Line::from("  re-staged when the source file is modified."),
        Line::from("  File watching continues until you exit LazyDraft."),
        Line::from(""),
        Line::from("General:"),
        Line::from("  h/F1          Toggle this help"),
        Line::from("  q/Esc         Quit application"),
        Line::from(""),
        Line::from("Indicators:"),
        Line::from("  ●             Active project"),
        Line::from("  ⚠             Unconfigured project"),
        Line::from("  📝            Draft writing"),
        Line::from("  ✅            Published writing"),
        Line::from("  🚀            Staged writing (auto-staging enabled)"),
        Line::from(""),
        Line::from("Press h or Esc to close this help"),
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text))
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(theme.border_style())
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(help_paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn draw_stage_confirm_popup(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(50, 25, f.size());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    let popup = Paragraph::new("Stage this writing?\n\nThis will transfer the content to the target location.\n\ny: Yes | n: No")
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Stage Writing")
                .borders(Borders::ALL)
                .border_style(theme.warning_style())
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(popup, area);
}

fn draw_revert_confirm_popup(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(50, 25, f.size());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    let popup = Paragraph::new("Revert staging for this writing?\n\nThis will remove it from the staged list.\n\ny: Yes | n: No")
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Revert Staging")
                .borders(Borders::ALL)
                .border_style(theme.danger_style())
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(popup, area);
}

fn draw_operation_result_popup(f: &mut Frame, success: bool, message: &str, theme: &Theme) {
    let area = centered_rect(60, 20, f.size());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    let (title, border_style, text_color) = if success {
        ("Success", theme.success_style(), theme.text)
    } else {
        ("Error", theme.danger_style(), theme.text)
    };
    
    let popup_text = format!("{}\n\nPress Enter to continue", message);
    
    let popup = Paragraph::new(popup_text)
        .style(Style::default().fg(text_color))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style)
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(popup, area);
}

fn format_relative_time(timestamp: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(timestamp) {
        Ok(dt) => {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
            
            if duration.num_days() > 0 {
                format!("{}d ago", duration.num_days())
            } else if duration.num_hours() > 0 {
                format!("{}h ago", duration.num_hours())
            } else if duration.num_minutes() > 0 {
                format!("{}m ago", duration.num_minutes())
            } else {
                "Just now".to_string()
            }
        }
        Err(_) => timestamp.to_string(),
    }
} 
