use ratatui::prelude::*;

pub struct Theme {
    pub normal_style: Style,
    pub header_style: Style,
    pub highlight_style: Style,
    pub border_style: Style,
    pub label_style: Style,
    pub value_style: Style,
    pub key_style: Style,
    pub success_style: Style,
    pub warning_style: Style,
    pub error_style: Style,
    pub na_style: Style,
    pub attention_style: Style,
}

pub fn get_theme(dark_mode: bool) -> Theme {
    if dark_mode {
        Theme {
            normal_style: Style::default().fg(Color::White),
            header_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            highlight_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            border_style: Style::default().fg(Color::DarkGray),
            label_style: Style::default().fg(Color::Gray),
            value_style: Style::default().fg(Color::White),
            key_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            success_style: Style::default().fg(Color::Green),
            warning_style: Style::default().fg(Color::Yellow),
            error_style: Style::default().fg(Color::Red),
            na_style: Style::default().fg(Color::DarkGray),
            attention_style: Style::default().fg(Color::Magenta),
        }
    } else {
        Theme {
            normal_style: Style::default().fg(Color::Black),
            header_style: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            highlight_style: Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            border_style: Style::default().fg(Color::Gray),
            label_style: Style::default().fg(Color::DarkGray),
            value_style: Style::default().fg(Color::Black),
            key_style: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            success_style: Style::default().fg(Color::Green),
            warning_style: Style::default().fg(Color::Yellow),
            error_style: Style::default().fg(Color::Red),
            na_style: Style::default().fg(Color::Gray),
            attention_style: Style::default().fg(Color::Magenta),
        }
    }
}

impl Theme {
    pub fn get_status_style(&self, status: crate::telemetry::ValueStatus) -> Style {
        use crate::telemetry::ValueStatus;
        match status {
            ValueStatus::Normal => self.success_style,
            ValueStatus::Warning => self.warning_style,
            ValueStatus::Critical => self.error_style,
        }
    }

    pub fn get_telemetry_style(
        &self,
        status: crate::telemetry::ValueStatus,
        is_blackhole: bool,
    ) -> Style {
        if is_blackhole {
            self.attention_style
        } else {
            self.get_status_style(status)
        }
    }
}
