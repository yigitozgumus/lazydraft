use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy)]
pub struct Theme {
    pub accent: Color,
    pub accent_soft: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            accent: Color::Rgb(238, 174, 96),
            accent_soft: Color::Rgb(206, 157, 98),
            border: Color::Rgb(62, 64, 68),
            text: Color::Rgb(230, 230, 230),
            muted: Color::Rgb(140, 140, 140),
            success: Color::Rgb(94, 201, 154),
            warning: Color::Rgb(240, 191, 76),
            danger: Color::Rgb(232, 108, 97),
            highlight_bg: Color::Rgb(35, 39, 46),
            highlight_fg: Color::Rgb(240, 240, 240),
        }
    }

    pub fn header_style(&self) -> Style {
        Style::default().fg(self.accent).add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn danger_style(&self) -> Style {
        Style::default().fg(self.danger)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .bg(self.highlight_bg)
            .fg(self.highlight_fg)
            .add_modifier(Modifier::BOLD)
    }
}
