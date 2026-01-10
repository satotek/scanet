use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};

pub fn render_device_list(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["IP Address", "MAC Address", "Vendor", "Hostname", "Response"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .height(1);

    let rows: Vec<Row> = app
        .devices
        .iter()
        .enumerate()
        .map(|(i, device)| {
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if device.is_new {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            Row::new(vec![
                device.ip.to_string(),
                device.mac.clone().unwrap_or_else(|| "-".to_string()),
                device.vendor.clone().unwrap_or_else(|| "-".to_string()),
                device.hostname.clone().unwrap_or_else(|| "-".to_string()),
                format!("{:.1}ms", device.response_time_ms()),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Length(16),
        ratatui::layout::Constraint::Length(18),
        ratatui::layout::Constraint::Length(20),
        ratatui::layout::Constraint::Min(15),
        ratatui::layout::Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Devices"))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = TableState::default();
    if !app.devices.is_empty() {
        state.select(Some(app.selected_index));
    }

    frame.render_stateful_widget(table, area, &mut state);
}
