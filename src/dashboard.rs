use std::io;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};

use crate::config::{get_project_manager, ProjectConfig, ProjectManager};
use crate::writing::create_writing_list;

pub struct Dashboard {
    project_manager: ProjectManager,
    projects: Vec<ProjectConfig>,
    active_project: Option<String>,
    selected_index: usize,
    list_state: ListState,
    show_help: bool,
    last_update: Instant,
    project_stats: Vec<ProjectStats>,
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
            let (draft_count, total_files) = if project.config.source_dir.is_some() {
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
        terminal.draw(|f| ui(f, dashboard))?;
        
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('h') | KeyCode::F(1) => {
                            dashboard.show_help = !dashboard.show_help;
                        }
                        KeyCode::Char('r') | KeyCode::F(5) => {
                            if let Err(e) = dashboard.refresh_data() {
                                // Could show error in UI, for now just continue
                                eprintln!("Refresh error: {}", e);
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            dashboard.next_project();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            dashboard.previous_project();
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if let Err(e) = dashboard.switch_to_selected_project() {
                                eprintln!("Switch error: {}", e);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Auto-refresh every 30 seconds
        if dashboard.last_update.elapsed() > Duration::from_secs(30) {
            if let Err(e) = dashboard.refresh_data() {
                eprintln!("Auto-refresh error: {}", e);
            }
        }
    }
}

fn ui(f: &mut Frame, dashboard: &Dashboard) {
    if dashboard.show_help {
        draw_help_popup(f);
        return;
    }
    
    let size = f.area();
    
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
    let header = Paragraph::new("LazyDraft Dashboard")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);
    
    // Main content layout
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Projects list
            Constraint::Percentage(50),  // Project details & stats
        ])
        .split(chunks[1]);
    
    // Projects list
    draw_projects_list(f, dashboard, main_chunks[0]);
    
    // Right panel layout
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Project details
            Constraint::Percentage(40),  // Overall stats
        ])
        .split(main_chunks[1]);
    
    // Project details
    draw_project_details(f, dashboard, right_chunks[0]);
    
    // Overall stats
    draw_overall_stats(f, dashboard, right_chunks[1]);
    
    // Footer
    draw_footer(f, chunks[2]);
}

fn draw_projects_list(f: &mut Frame, dashboard: &Dashboard, area: Rect) {
    let items: Vec<ListItem> = dashboard
        .project_stats
        .iter()
        .enumerate()
        .map(|(i, stats)| {
            let project = &dashboard.projects[i];
            let is_configured = project.config.source_dir.is_some() && project.config.target_dir.is_some();
            
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
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else if !is_configured {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default()
                    }
                ),
                Span::styled(
                    draft_info,
                    Style::default().fg(Color::Yellow)
                ),
            ]);
            
            ListItem::new(line)
        })
        .collect();
    
    let projects_list = List::new(items)
        .block(
            Block::default()
                .title("Projects")
                .borders(Borders::ALL)
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▶ ");
    
    f.render_stateful_widget(projects_list, area, &mut dashboard.list_state.clone());
}

fn draw_project_details(f: &mut Frame, dashboard: &Dashboard, area: Rect) {
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
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Gray)
                    }
                ),
            ]),
        ]);
        
        // Check if project is configured
        let is_configured = project.config.source_dir.is_some() && project.config.target_dir.is_some();
        
        if is_configured {
            lines.extend(vec![
                Line::from(vec![
                    Span::styled("Drafts: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        stats.draft_count.to_string(),
                        if stats.draft_count > 0 {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::Green)
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
                    Span::raw(project.config.source_dir.as_deref().unwrap_or("not set")),
                ]),
                Line::from(vec![
                    Span::styled("Target: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(project.config.target_dir.as_deref().unwrap_or("not set")),
                ]),
            ]);
        } else {
            lines.extend(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("⚠ Configuration Required", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from("This project needs to be configured before use."),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Run: "),
                    Span::styled(
                        format!("lazydraft config --edit --project {}", stats.name),
                        Style::default().fg(Color::Cyan)
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Source: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("not set", Style::default().fg(Color::Red)),
                ]),
                Line::from(vec![
                    Span::styled("Target: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("not set", Style::default().fg(Color::Red)),
                ]),
            ]);
        }
        
        Text::from(lines)
    } else {
        Text::from("No project selected")
    };
    
    let details = Paragraph::new(content)
        .block(
            Block::default()
                .title("Project Details")
                .borders(Borders::ALL)
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(details, area);
}

fn draw_overall_stats(f: &mut Frame, dashboard: &Dashboard, area: Rect) {
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
                Style::default().fg(Color::Green)
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Drafts: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                total_drafts.to_string(),
                if total_drafts > 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
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
            Span::raw(format!("{}s ago", dashboard.last_update.elapsed().as_secs())),
        ]),
    ]);
    
    let stats = Paragraph::new(content)
        .block(
            Block::default()
                .title("Overall Statistics")
                .borders(Borders::ALL)
        );
    
    f.render_widget(stats, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let footer_text = "↑↓/jk: Navigate | Enter/Space: Switch | r/F5: Refresh | h/F1: Help | q/Esc: Quit";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(footer, area);
}

fn draw_help_popup(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    
    let help_text = Text::from(vec![
        Line::from(vec![
            Span::styled("LazyDraft Dashboard Help", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  ↑↓ or j/k    - Move up/down in project list"),
        Line::from("  Enter/Space  - Switch to selected project"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  r or F5      - Refresh project data"),
        Line::from("  h or F1      - Toggle this help"),
        Line::from("  q or Esc     - Quit dashboard"),
        Line::from(""),
        Line::from("Features:"),
        Line::from("  • Real-time project status"),
        Line::from("  • Draft count tracking"),
        Line::from("  • Quick project switching"),
        Line::from("  • Auto-refresh every 30s"),
        Line::from(""),
        Line::from("Press h or F1 to close this help"),
    ]);
    
    let help_popup = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black))
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(Clear, area);
    f.render_widget(help_popup, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
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