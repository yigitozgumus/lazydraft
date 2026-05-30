use std::path::Path;
use std::time::Duration;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::dashboard::{Dashboard, ViewMode, PopupType};
use crate::tui::Theme;

// ── Main UI ─────────────────────────────────────────────────────────────────

pub fn ui(f: &mut Frame, dashboard: &Dashboard, projects_list_state: &mut ListState, writings_list_state: &mut ListState) {
    let theme = Theme::default();
    if dashboard.show_help {
        draw_help_popup(f, &theme);
        return;
    }

    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    // Header
    let active_project_name = dashboard.active_project.as_deref().unwrap_or("none");
    let auto_stage = if dashboard.auto_stage_enabled { "ON" } else { "OFF" };
    let refresh_secs = dashboard.last_update.elapsed().as_secs();

    let header_title = match dashboard.view_mode {
        ViewMode::Projects => {
            format!(
                "LazyDraft  |  Active: {}  |  Auto-stage: {}  |  {}s ago",
                active_project_name, auto_stage, refresh_secs
            )
        }
        ViewMode::Writings => {
            if let Some(project) = dashboard.projects.get(dashboard.selected_index) {
                format!(
                    "LazyDraft · {}  |  Active: {}  |  Auto-stage: {}  |  {}s ago",
                    project.name, active_project_name, auto_stage, refresh_secs
                )
            } else {
                format!(
                    "LazyDraft Writings  |  Auto-stage: {}  |  {}s ago",
                    auto_stage, refresh_secs
                )
            }
        }
    };

    let header = Paragraph::new(header_title)
        .style(theme.header_style())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(theme.border_style()));
    f.render_widget(header, chunks[0]);

    match dashboard.view_mode {
        ViewMode::Projects => draw_projects_view(f, dashboard, projects_list_state, chunks[1], &theme),
        ViewMode::Writings => draw_writings_view(f, dashboard, writings_list_state, chunks[1], &theme),
    }

    draw_footer(f, chunks[2], dashboard, &theme);

    match &dashboard.popup_type {
        PopupType::StageConfirm => draw_stage_confirm_popup(f, &theme),
        PopupType::RevertConfirm => draw_revert_confirm_popup(f, &theme),
        PopupType::OperationResult { success, message } => {
            draw_operation_result_popup(f, *success, message, &theme)
        }
        PopupType::None => {}
    }
}

// ── Projects View ───────────────────────────────────────────────────────────

fn draw_projects_view(f: &mut Frame, dashboard: &Dashboard, list_state: &mut ListState, area: Rect, theme: &Theme) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_projects_list(f, dashboard, list_state, main_chunks[0], theme);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[1]);

    draw_project_details(f, dashboard, right_chunks[0], theme);
    draw_overall_stats(f, dashboard, right_chunks[1], theme);
}

fn draw_projects_list(f: &mut Frame, dashboard: &Dashboard, list_state: &mut ListState, area: Rect, theme: &Theme) {
    let items: Vec<ListItem> = if dashboard.project_stats.is_empty() {
        vec![ListItem::new(Line::from(Span::styled("No projects found", theme.muted_style())))]
    } else {
        dashboard.project_stats.iter().enumerate().map(|(i, stats)| {
            let project = &dashboard.projects[i];
            let is_configured = project.config.get_source_dir().is_some()
                && project.config.get_target_dir().is_some();

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
        }).collect()
    };

    let list = List::new(items)
        .block(Block::default().title("Projects").borders(Borders::ALL).border_style(theme.border_style()))
        .highlight_style(theme.highlight_style())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, list_state);
}

// ── Writings View ───────────────────────────────────────────────────────────

fn draw_writings_view(f: &mut Frame, dashboard: &Dashboard, list_state: &mut ListState, area: Rect, theme: &Theme) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_writings_list(f, dashboard, list_state, main_chunks[0], theme);
    draw_writing_details(f, dashboard, main_chunks[1], theme);
}

fn draw_writings_list(f: &mut Frame, dashboard: &Dashboard, list_state: &mut ListState, area: Rect, theme: &Theme) {
    let items: Vec<ListItem> = if dashboard.writings.is_empty() {
        vec![ListItem::new(Line::from(Span::styled("No writings found", theme.muted_style())))]
    } else {
        dashboard.writings.iter().map(|writing| {
            let status_tag = if writing.is_draft { " DRAFT " } else { " PUB " };
            let status_bg = if writing.is_draft { theme.warning } else { theme.success };

            let staged_tag = if dashboard.staged_writings.contains(&writing.path) {
                if dashboard.auto_stage_enabled { " [AUTO]" } else { " [STAGED]" }
            } else {
                ""
            };

            let date_str = writing.publish_date
                .map(|d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "--------".to_string());

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", status_tag),
                    Style::default().fg(theme.highlight_bg).bg(status_bg).add_modifier(Modifier::BOLD),
                ),
                Span::raw(&writing.title),
                Span::styled(staged_tag, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {:>10}", date_str), Style::default().fg(theme.muted)),
            ]);
            ListItem::new(line)
        }).collect()
    };

    let writings_count = dashboard.writings.len();
    let staged_count = dashboard.staged_writings.len();
    let auto_stage_status = if dashboard.auto_stage_enabled { "ON" } else { "OFF" };
    let title = if staged_count > 0 {
        format!("Writings ({} total, {} staged, auto-stage: {})", writings_count, staged_count, auto_stage_status)
    } else {
        format!("Writings ({} total)", writings_count)
    };

    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL).border_style(theme.border_style()))
        .highlight_style(theme.highlight_style())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, list_state);
}

fn draw_writing_details(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    let details = if let Some(writing) = dashboard.writings.get(dashboard.selected_writings_index) {
        let status = if writing.is_draft { "Draft" } else { "Published" };
        let status_color = if writing.is_draft { theme.warning } else { theme.success };
        let is_staged = dashboard.staged_writings.contains(&writing.path);

        let publish_date_str = writing.publish_date
            .map(|d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "Not set".to_string());

        let file_size_str = std::fs::metadata(&writing.path).ok().map(|m| {
            let len = m.len();
            if len < 1024 { format!("{} B", len) }
            else if len < 1024 * 1024 { format!("{:.1} KB", len as f64 / 1024.0) }
            else { format!("{:.1} MB", len as f64 / (1024.0 * 1024.0)) }
        }).unwrap_or_else(|| "Unknown".to_string());

        let staging_info = if is_staged {
            let auto = if dashboard.auto_stage_enabled { " (auto)" } else { "" };
            format!("Staged{}", auto)
        } else {
            String::new()
        };

        let mut lines = vec![
            Line::from(vec![Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&writing.title)]),
            Line::from(""),
            Line::from(vec![Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled(status, Style::default().fg(status_color).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled("Date: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(publish_date_str.clone())]),
            Line::from(vec![Span::styled("Size: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(file_size_str.clone())]),
            Line::from(vec![Span::styled("Staged: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled(if is_staged { "Yes" } else { "No" }, if is_staged { theme.success_style() } else { theme.muted_style() })]),
            Line::from(""),
            Line::from(vec![Span::styled("Path: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&writing.path)]),
        ];

        if !staging_info.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(staging_info, theme.success_style())));
        }

        Paragraph::new(Text::from(lines))
            .block(Block::default().title("Writing Details").borders(Borders::ALL).border_style(theme.border_style()))
            .wrap(Wrap { trim: true })
    } else {
        Paragraph::new(Text::from(Line::from(Span::styled("No writing selected", theme.muted_style()))))
            .block(Block::default().title("Writing Details").borders(Borders::ALL).border_style(theme.border_style()))
            .wrap(Wrap { trim: true })
    };
    f.render_widget(details, area);
}

// ── Project Details ─────────────────────────────────────────────────────────

fn path_exists_indicator(path: &str) -> &'static str {
    if path.is_empty() || path == "not set" { return " ∅"; }
    if Path::new(path).exists() { " ✅" } else { " ❌" }
}

fn draw_project_details(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    let details = if let Some(stats) = dashboard.project_stats.get(dashboard.selected_index) {
        let project = &dashboard.projects[dashboard.selected_index];
        let mut lines = vec![
            Line::from(vec![Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&stats.name)]),
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
                Span::styled(if stats.is_active { "Active" } else { "Inactive" },
                    if stats.is_active { theme.success_style() } else { theme.muted_style() }),
            ]),
        ]);

        let is_configured = project.config.get_source_dir().is_some() && project.config.get_target_dir().is_some();

        if is_configured {
            let source = project.config.get_source_dir().unwrap_or_default();
            let target = project.config.get_target_dir().unwrap_or_default();

            lines.extend(vec![
                Line::from(vec![Span::styled("Drafts: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(stats.draft_count.to_string(), if stats.draft_count > 0 { theme.warning_style() } else { theme.success_style() })]),
                Line::from(vec![Span::styled("Total Files: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(stats.total_files.to_string())]),
                Line::from(vec![Span::styled("Last Activity: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&stats.last_activity)]),
                Line::from(""),
                Line::from(vec![Span::styled("Source:", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(path_exists_indicator(&source), Style::default().fg(theme.text)),
                    Span::raw(source)]),
                Line::from(vec![Span::styled("Target:", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(path_exists_indicator(&target), Style::default().fg(theme.text)),
                    Span::raw(target)]),
            ]);
        } else {
            lines.extend(vec![
                Line::from(""),
                Line::from(Span::styled("Configuration Required", theme.danger_style().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from("This project needs to be configured before use."),
                Line::from(""),
                Line::from(vec![Span::raw("Run: "), Span::styled(
                    format!("lazydraft config --edit --project {}", stats.name), Style::default().fg(theme.accent))]),
                Line::from(""),
                Line::from(vec![Span::styled("Source: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled("not set", theme.danger_style())]),
                Line::from(vec![Span::styled("Target: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled("not set", theme.danger_style())]),
            ]);
        }

        Paragraph::new(Text::from(lines))
            .block(Block::default().title("Project Details").borders(Borders::ALL).border_style(theme.border_style()))
            .wrap(Wrap { trim: true })
    } else {
        Paragraph::new(Text::from(Line::from(Span::styled("No project selected", theme.muted_style()))))
            .block(Block::default().title("Project Details").borders(Borders::ALL).border_style(theme.border_style()))
            .wrap(Wrap { trim: true })
    };
    f.render_widget(details, area);
}

// ── Overall Stats ───────────────────────────────────────────────────────────

fn draw_overall_stats(f: &mut Frame, dashboard: &Dashboard, area: Rect, theme: &Theme) {
    let total_projects = dashboard.projects.len();
    let total_drafts: usize = dashboard.project_stats.iter().map(|s| s.draft_count).sum();
    let total_files: usize = dashboard.project_stats.iter().map(|s| s.total_files).sum();
    let active_projects = dashboard.project_stats.iter().filter(|s| s.is_active).count();

    let content = Text::from(vec![
        Line::from(vec![Span::styled("Total Projects: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(total_projects.to_string())]),
        Line::from(vec![Span::styled("Active Projects: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(active_projects.to_string(), theme.success_style())]),
        Line::from(vec![Span::styled("Total Drafts: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(total_drafts.to_string(), if total_drafts > 0 { theme.warning_style() } else { theme.success_style() })]),
        Line::from(vec![Span::styled("Total Files: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(total_files.to_string())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Last Update: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}s ago", dashboard.last_update.elapsed().as_secs()), theme.muted_style()),
        ]),
    ]);

    let stats = Paragraph::new(content)
        .block(Block::default().title("Overall Statistics").borders(Borders::ALL).border_style(theme.border_style()));
    f.render_widget(stats, area);
}

// ── Footer ───────────────────────────────────────────────────────────────────

fn draw_footer(f: &mut Frame, area: Rect, dashboard: &Dashboard, theme: &Theme) {
    let keybindings = match dashboard.view_mode {
        ViewMode::Projects => "q:Quit h:Help r:Refresh ↑↓:Navigate →:Writings Enter:Switch".to_string(),
        ViewMode::Writings => {
            let auto_stage = if dashboard.auto_stage_enabled { "ON" } else { "OFF" };
            format!("q:Quit h:Help r:Refresh ↑↓:Navigate ←:Back s:Stage u:Revert a:Auto({})", auto_stage)
        }
    };

    let message_line = match &dashboard.last_message {
        Some((msg, success, ts)) if ts.elapsed() < Duration::from_secs(5) => {
            let style = if *success { theme.success_style() } else { theme.danger_style() };
            Some(Span::styled(msg.as_str(), style))
        }
        _ => None,
    };

    let footer_text = if let Some(msg_span) = message_line {
        Text::from(Line::from(vec![
            Span::styled(&keybindings, theme.muted_style()),
            Span::raw("  |  "),
            msg_span,
        ]))
    } else {
        Text::from(Line::from(Span::styled(&keybindings, theme.muted_style())))
    };

    let footer = Paragraph::new(footer_text)
        .style(theme.muted_style())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(theme.border_style()));
    f.render_widget(footer, area);
}

// ── Help Popup ───────────────────────────────────────────────────────────────

fn draw_help_popup(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(70, 60, f.size());
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
        Line::from("  DRAFT         Draft writing"),
        Line::from("  PUB           Published writing"),
        Line::from("  [AUTO]        Staged with auto-staging enabled"),
        Line::from("  [STAGED]      Staged without auto-staging"),
        Line::from(""),
        Line::from("Press h or Esc to close this help"),
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text))
        .block(Block::default().title("Help").borders(Borders::ALL).border_style(theme.border_style()))
        .wrap(Wrap { trim: true });
    f.render_widget(help, area);
}

// ── Confirm / Result Popups ─────────────────────────────────────────────────

fn draw_stage_confirm_popup(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(50, 25, f.size());
    f.render_widget(Clear, area);

    let popup = Paragraph::new("Stage this writing?\n\nThis will transfer the content to the target location.\n\ny: Yes | n: No")
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center)
        .block(Block::default().title("Stage Writing").borders(Borders::ALL).border_style(theme.warning_style()))
        .wrap(Wrap { trim: true });
    f.render_widget(popup, area);
}

fn draw_revert_confirm_popup(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(50, 25, f.size());
    f.render_widget(Clear, area);

    let popup = Paragraph::new("Revert staging for this writing?\n\nThis will remove it from the staged list.\n\ny: Yes | n: No")
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center)
        .block(Block::default().title("Revert Staging").borders(Borders::ALL).border_style(theme.danger_style()))
        .wrap(Wrap { trim: true });
    f.render_widget(popup, area);
}

fn draw_operation_result_popup(f: &mut Frame, success: bool, message: &str, theme: &Theme) {
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);

    let (title, border_style, text_color) = if success {
        ("Success", theme.success_style(), theme.text)
    } else {
        ("Error", theme.danger_style(), theme.text)
    };

    let popup = Paragraph::new(format!("{}\n\nPress Enter to continue", message))
        .style(Style::default().fg(text_color))
        .alignment(Alignment::Center)
        .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
        .wrap(Wrap { trim: true });
    f.render_widget(popup, area);
}

// ── Helpers ─────────────────────────────────────────────────────────────────

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

pub fn format_relative_time(timestamp: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(timestamp) {
        Ok(dt) => {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
            if duration.num_days() > 0 { format!("{}d ago", duration.num_days()) }
            else if duration.num_hours() > 0 { format!("{}h ago", duration.num_hours()) }
            else if duration.num_minutes() > 0 { format!("{}m ago", duration.num_minutes()) }
            else { "Just now".to_string() }
        }
        Err(_) => timestamp.to_string(),
    }
}
