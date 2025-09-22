use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::ui::theme::Theme;

pub fn draw_telemetry_tab(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let telemetry_data = app.get_telemetry_data();
    let selected = app.get_selected_device();

    let header = Row::new(vec![
        "#",
        "Core Voltage (V)",
        "Core Current (A)",
        "AICLK (MHz)",
        "Core Power (W)",
        "Core Temp (°C)",
        "Heartbeat",
    ])
    .style(theme.header_style)
    .bottom_margin(1);

    let rows: Vec<Row> = telemetry_data
        .iter()
        .enumerate()
        .map(|(idx, data)| {
            let (tel, limits) = (&data.telemetry, &data.limits);
            let is_blackhole = data.device_info.board_type.is_blackhole();

            let cells = vec![
                Cell::from(data.device_info.index.to_string()),
                Cell::from(format!("{:.3}/{:.3}", tel.voltage, limits.vdd_max))
                    .style(theme.get_telemetry_style(tel.voltage_status(limits), is_blackhole)),
                Cell::from(format!("{:.1}/{:.1}", tel.current, limits.tdc_limit))
                    .style(theme.get_telemetry_style(tel.current_status(limits), is_blackhole)),
                Cell::from(format!("{}/{}", tel.aiclk, limits.asic_fmax))
                    .style(theme.get_telemetry_style(tel.aiclk_status(limits), is_blackhole)),
                Cell::from(format!("{:.1}/{:.1}", tel.power, limits.tdp_limit))
                    .style(theme.get_telemetry_style(tel.power_status(limits), is_blackhole)),
                Cell::from(format!("{:.1}/{:.1}", tel.temperature, limits.thm_limit))
                    .style(theme.get_telemetry_style(tel.temperature_status(limits), is_blackhole)),
                Cell::from(get_heartbeat_symbol(
                    tel.calculate_heartbeat(data.device_info.board_type.chip_arch()),
                ))
                .style(theme.success_style),
            ];

            Row::new(cells).style(if idx == selected {
                theme.highlight_style
            } else {
                theme.normal_style
            })
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Length(3),
            Constraint::Min(16),
            Constraint::Min(16),
            Constraint::Min(12),
            Constraint::Min(14),
            Constraint::Min(14),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_style)
            .title(" Real-time Telemetry (Updates: 100ms) ")
            .title_alignment(Alignment::Center),
    )
    .highlight_style(theme.highlight_style)
    .highlight_symbol(">> ");

    frame.render_stateful_widget(
        table,
        area,
        &mut TableState::default().with_selected(Some(selected)),
    );
}

fn get_heartbeat_symbol(heartbeat: u32) -> &'static str {
    let patterns = ["●∙∙", "∙●∙", "∙∙●", "∙●∙"];
    patterns[heartbeat as usize % patterns.len()]
}
