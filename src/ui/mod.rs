mod detail;
mod device_list;
mod help;
mod range_input;

pub use device_list::render_device_list;

use crate::app::{App, ScanState, View};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header
    let header = Paragraph::new(format!("scanet - LAN Device Scanner    [{}]", app.scan_range_cidr()))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Device list
    render_device_list(frame, app, chunks[1]);

    // Status bar
    let (scan_status, status_style) = match app.scan_state {
        ScanState::Scanning => {
            let ip_info = match app.current_scanning_ip {
                Some(ip) => format!(" [{}]", ip),
                None => String::new(),
            };
            let progress = format!(
                "{} Scanning{} {}/{} ({}%)",
                app.spinner_char(),
                ip_info,
                app.scanned_count,
                app.total_count,
                app.progress_percent()
            );
            (progress, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        }
        ScanState::Waiting => {
            let remaining = app.wait_remaining_secs();
            let status = format!(
                "{} Waiting... {}s",
                app.spinner_char(),
                remaining
            );
            (status, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        }
        ScanState::Idle => {
            let status = match app.time_since_last_scan() {
                Some(secs) => format!("Last scan: {}s ago", secs),
                None => String::from("Not scanned yet"),
            };
            (status, Style::default().fg(Color::White))
        }
    };

    let continuous_indicator = if app.continuous_mode {
        format!("ON ({}s)", app.scan_interval_secs)
    } else {
        String::from("OFF")
    };

    let status_text = match app.scan_state {
        ScanState::Scanning | ScanState::Waiting => {
            format!(
                "Devices: {} | {} | [C]ontinuous: {} | [Q]uit",
                app.devices.len(),
                scan_status,
                continuous_indicator
            )
        }
        ScanState::Idle => {
            format!(
                "Devices: {} | {} | {} | [S]can [C]ontinuous: {} [Q]uit",
                app.devices.len(),
                scan_status,
                &app.status_message,
                continuous_indicator
            )
        }
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, chunks[2]);

    // Render overlays based on current view
    match app.current_view {
        View::Help => {
            help::render_help(frame);
        }
        View::DeviceDetail => {
            if let Some(device) = app.selected_device() {
                detail::render_device_detail(frame, device);
            }
        }
        View::RangeInput => {
            range_input::render_range_input(frame, app);
        }
        View::Main => {}
    }
}
