use ratatui::prelude::*;
use ratatui::widgets::*;

mod device_info;
mod firmware;
mod help;
mod telemetry;
mod theme;

use crate::app::App;
use device_info::draw_device_info_tab;
use firmware::draw_firmware_tab;
use help::draw_help_modal;
use telemetry::draw_telemetry_tab;
use theme::get_theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    DeviceInfo,
    Telemetry,
    Firmware,
}

pub fn draw_ui(frame: &mut Frame, app: &App) {
    let theme = get_theme(app.is_dark_mode());
    let chunks = create_layout(frame.size(), app.should_show_sidebar());

    draw_header(frame, chunks[0], &theme);
    draw_tabs(frame, chunks[1], app.get_current_tab(), &theme);

    let content_area = if app.should_show_sidebar() {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(chunks[2]);

        draw_sidebar(frame, content_chunks[1], app, &theme);

        content_chunks[0]
    } else {
        chunks[2]
    };

    match app.get_current_tab() {
        Tab::DeviceInfo => draw_device_info_tab(frame, content_area, app, &theme),
        Tab::Telemetry => draw_telemetry_tab(frame, content_area, app, &theme),
        Tab::Firmware => draw_firmware_tab(frame, content_area, app, &theme),
    }

    draw_footer(frame, chunks[3], &theme);

    if app.should_show_help() {
        draw_help_modal(frame, frame.size(), &theme);
    }
}

fn create_layout(area: Rect, _show_sidebar: bool) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area)
        .to_vec()
}

fn draw_header(frame: &mut Frame, area: Rect, theme: &theme::Theme) {
    let header_text = format!("TT-SMI v{}", env!("CARGO_PKG_VERSION"));
    let header = Paragraph::new(header_text)
        .style(theme.header_style)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style)
                .title(" Tenstorrent System Management Interface ")
                .title_alignment(Alignment::Center),
        );
    frame.render_widget(header, area);
}

fn draw_tabs(frame: &mut Frame, area: Rect, current_tab: Tab, theme: &theme::Theme) {
    let titles = vec!["[1] Device Info", "[2] Telemetry", "[3] Firmware"];
    let selected_index = match current_tab {
        Tab::DeviceInfo => 0,
        Tab::Telemetry => 1,
        Tab::Firmware => 2,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style),
        )
        .select(selected_index)
        .style(theme.normal_style)
        .highlight_style(theme.highlight_style);

    frame.render_widget(tabs, area);
}

fn draw_sidebar(frame: &mut Frame, area: Rect, _app: &App, theme: &theme::Theme) {
    let info = crate::utils::get_host_info();
    let lines = vec![
        Line::from(vec![Span::styled(
            "Host Info",
            theme.header_style.add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("OS: ", theme.label_style),
            Span::styled(&info.os, theme.value_style),
        ]),
        Line::from(vec![
            Span::styled("Kernel: ", theme.label_style),
            Span::styled(&info.kernel, theme.value_style),
        ]),
        Line::from(vec![
            Span::styled("Driver: ", theme.label_style),
            Span::styled(&info.driver, theme.value_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("tt-smi: ", theme.label_style),
            Span::styled(env!("CARGO_PKG_VERSION"), theme.value_style),
        ]),
    ];

    let sidebar = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style)
                .title(" System Information ")
                .title_alignment(Alignment::Center),
        )
        .style(theme.normal_style)
        .wrap(Wrap { trim: true });

    frame.render_widget(sidebar, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, theme: &theme::Theme) {
    let shortcuts = [
        ("q", "Quit"),
        ("h", "Help"),
        ("1-3", "Switch tabs"),
        ("↑↓/jk", "Navigate"),
        ("d", "Dark mode"),
        ("c", "Toggle sidebar"),
    ];

    let spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(format!("[{key}]"), theme.key_style),
                Span::styled(format!(" {desc} "), theme.normal_style),
            ]
        })
        .collect();

    let footer = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style),
        )
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
