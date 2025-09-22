use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::ui::theme::Theme;

pub fn draw_firmware_tab(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let telemetry_data = app.get_telemetry_data();
    let selected = app.get_selected_device();

    let header = Row::new(vec![
        "#",
        "FW Bundle Version",
        "TT-Flash Version",
        "CM FW Version",
        "CM FW Date",
        "ETH FW Version",
        "BM BL Version",
        "BM App Version",
    ])
    .style(theme.header_style)
    .bottom_margin(1);

    let rows: Vec<Row> = telemetry_data
        .iter()
        .enumerate()
        .map(|(idx, data)| {
            let fw = &data.firmware_info;
            let is_selected = idx == selected;

            let row = Row::new(vec![
                Cell::from(format!("{}", data.device_info.index)),
                Cell::from(fw.fw_bundle_version.clone()),
                Cell::from(fw.tt_flash_version.clone()),
                Cell::from(fw.cm_fw_version.clone()),
                Cell::from(fw.cm_fw_date.clone()),
                Cell::from(fw.eth_fw_version.clone()),
                Cell::from(fw.bm_bl_version.clone()),
                Cell::from(fw.bm_app_version.clone()),
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
            Constraint::Min(16),
            Constraint::Min(15),
            Constraint::Min(14),
            Constraint::Min(12),
            Constraint::Min(14),
            Constraint::Min(13),
            Constraint::Min(14),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_style)
            .title(" Firmware Information ")
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
