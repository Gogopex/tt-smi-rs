use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::telemetry::{DramStatus, PcieSpeed, PcieWidth};
use crate::ui::theme::Theme;

fn format_pcie_speed(current: PcieSpeed, max: PcieSpeed) -> String {
    match (current, max) {
        (PcieSpeed::NA, PcieSpeed::NA) => "N/A".to_string(),
        (current, max) if current == max => current.to_string(),
        (current, max) => format!("{current}/{max}"),
    }
}

fn format_pcie_width(current: PcieWidth, max: PcieWidth) -> String {
    match (current, max) {
        (PcieWidth::NA, PcieWidth::NA) => "N/A".to_string(),
        (current, max) if current == max => format!("x{current}"),
        (current, max) => format!("x{current}/x{max}"),
    }
}

pub fn draw_device_info_tab(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let telemetry_data = app.get_telemetry_data();
    let selected = app.get_selected_device();

    let header = Row::new(vec![
        "#",
        "Bus ID",
        "Board Type",
        "Board ID",
        "Coords",
        "DRAM Trained",
        "DRAM Speed",
        "Link Speed",
        "Link Width",
    ])
    .style(theme.header_style)
    .bottom_margin(1);

    let rows: Vec<Row> = telemetry_data
        .iter()
        .enumerate()
        .map(|(idx, data)| {
            let dev = &data.device_info;
            let is_selected = idx == selected;

            let dram_status_style = match dev.dram_status {
                DramStatus::Trained => theme.success_style,
                DramStatus::NotTrained => theme.error_style,
                DramStatus::Unknown => theme.warning_style,
            };

            let pcie_style = theme.get_status_style(dev.pcie_status());

            let row = Row::new(vec![
                Cell::from(format!("{}", dev.index)),
                Cell::from(dev.bus_id.clone()),
                Cell::from(dev.board_type.to_string()),
                Cell::from(dev.board_id.clone()),
                Cell::from(dev.coords.to_string()),
                Cell::from(dev.dram_status.to_string()).style(dram_status_style),
                Cell::from(format!("{} MHz", dev.dram_speed)),
                Cell::from(format_pcie_speed(dev.pcie_link_speed, dev.pcie_max_speed))
                    .style(pcie_style),
                Cell::from(format_pcie_width(dev.pcie_link_width, dev.pcie_max_width))
                    .style(pcie_style),
            ]);

            if is_selected {
                row.style(theme.highlight_style)
            } else {
                row.style(theme.normal_style)
            }
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Length(3),
            Constraint::Min(15),
            Constraint::Min(12),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(12),
            Constraint::Min(10),
            Constraint::Min(15),
            Constraint::Min(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_style)
            .title(" Device Information ")
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
