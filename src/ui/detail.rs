use crate::device::Device;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_device_detail(frame: &mut Frame, device: &Device) {
    let area = centered_rect(50, 50, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    // Pre-compute strings to avoid lifetime issues
    let ip_str = device.ip.to_string();
    let mac_str = device.mac.as_deref().unwrap_or("Unknown").to_string();
    let hostname_str = device.hostname.as_deref().unwrap_or("Unknown").to_string();
    let vendor_str = device.vendor.as_deref().unwrap_or("Unknown").to_string();
    let response_str = format!("{:.2} ms", device.response_time_ms());
    let last_seen_str = format!("{} seconds ago", device.last_seen.elapsed().as_secs());
    let status_str = if device.is_new { "New" } else { "Known" };

    let detail_text = vec![
        Line::from(Span::styled(
            "Device Details",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        make_detail_line("IP Address", &ip_str),
        Line::from(""),
        make_detail_line("MAC Address", &mac_str),
        Line::from(""),
        make_detail_line("Hostname", &hostname_str),
        Line::from(""),
        make_detail_line("Vendor", &vendor_str),
        Line::from(""),
        make_detail_line("Response Time", &response_str),
        Line::from(""),
        make_detail_line("Last Seen", &last_seen_str),
        Line::from(""),
        make_detail_line("Status", status_str),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or Enter to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let detail = Paragraph::new(detail_text)
        .block(
            Block::default()
                .title(format!(" {} ", device.ip))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(detail, area);
}

fn make_detail_line<'a>(label: &'static str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:<15}", label),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(value.to_string()),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
