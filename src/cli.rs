use colored::{ColoredString, Colorize};
use std::fmt::Display;

const LABEL_WIDTH: usize = 16;

fn dim(text: &str) -> ColoredString {
    text.bright_black()
}

pub fn header(title: &str, subtitle: Option<&str>) {
    let subtitle = subtitle.unwrap_or("");
    let width = title.len().max(subtitle.len());
    let inner = width + 2;
    let border = format!("+{}+", "-".repeat(inner));
    println!("{}", border.bright_black());
    println!("| {}{} |", title.cyan().bold(), " ".repeat(width - title.len()));
    if !subtitle.is_empty() {
        println!("| {}{} |", subtitle.bright_black(), " ".repeat(width - subtitle.len()));
    }
    println!("{}", border.bright_black());
}

pub fn section(title: &str) {
    println!("{}", title.cyan().bold());
    println!("{}", "-".repeat(title.len()).bright_black());
}

pub fn divider_with(width: usize) {
    println!("{}", "-".repeat(width).bright_black());
}

pub fn kv(label: &str, value: impl Display) {
    let padded = format!("{:<width$}", label, width = LABEL_WIDTH);
    println!("  {} {}", dim(&padded), value);
}

pub fn list_item(text: &str) {
    println!("  - {}", text);
}

pub fn info(message: &str) {
    println!("{} {}", "[INFO]".bright_blue().bold(), message);
}

pub fn success(message: &str) {
    println!("{} {}", "[OK]".green().bold(), message);
}

pub fn warn(message: &str) {
    eprintln!("{} {}", "[WARN]".yellow().bold(), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", "[ERR]".red().bold(), message);
}

pub fn blank_line() {
    println!();
}
