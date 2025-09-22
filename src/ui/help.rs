use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::ui::theme::Theme;

pub fn draw_help_modal(frame: &mut Frame, area: Rect, theme: &Theme) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "TT-SMI HELP MENU",
            theme.header_style.add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("TT-SMI is a TUI for monitoring Tenstorrent hardware."),
        Line::from(""),
        Line::from(vec![Span::styled("KEYBOARD SHORTCUTS", theme.header_style)]),
        Line::from(""),
        help_line("q", "Quit the application", theme),
        help_line("h", "Show this help menu", theme),
        help_line("1", "Switch to Device Info tab", theme),
        help_line("2", "Switch to Telemetry tab", theme),
        help_line("3", "Switch to Firmware tab", theme),
        help_line("↑/k", "Select previous device", theme),
        help_line("↓/j", "Select next device", theme),
        help_line("d", "Toggle dark/light mode", theme),
        help_line("c", "Toggle sidebar visibility", theme),
        Line::from(""),
        Line::from(vec![Span::styled(
            "COMMAND LINE OPTIONS",
            theme.header_style,
        )]),
        Line::from(""),
        help_line("tt-smi", "Launch interactive TUI (default)", theme),
        help_line("tt-smi list", "List all available devices", theme),
        help_line("tt-smi snapshot", "Export device data as JSON", theme),
        help_line("tt-smi reset <device>", "Reset a specific device", theme),
        Line::from(""),
        Line::from("Press any key to close this help menu."),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style)
                .title(" Help ")
                .title_alignment(Alignment::Center),
        )
        .style(theme.normal_style)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    let modal_width = 70;
    let modal_height = 30;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(
        x,
        y,
        modal_width.min(area.width),
        modal_height.min(area.height),
    );

    let overlay = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(overlay, area);

    frame.render_widget(Clear, modal_area);
    frame.render_widget(help_paragraph, modal_area);
}

fn help_line<'a>(key: &'a str, description: &'a str, theme: &Theme) -> Line<'a> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{key:<10}"), theme.key_style),
        Span::raw(format!(" - {description}")),
    ])
}
