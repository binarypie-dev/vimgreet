use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub foreground: Color,
    pub error: Color,
    pub success: Color,
    pub border: Color,
    pub muted: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Yellow,
            secondary: Color::Cyan,
            background: Color::Reset,
            foreground: Color::White,
            error: Color::Red,
            success: Color::Green,
            border: Color::DarkGray,
            muted: Color::DarkGray,
        }
    }
}

impl Theme {
    pub fn style(&self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }

    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    pub fn mode_style(&self, mode: &str) -> Style {
        let color = match mode {
            "NORMAL" => self.secondary,
            "INSERT" => self.success,
            "COMMAND" => self.primary,
            _ => self.foreground,
        };
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }
}
