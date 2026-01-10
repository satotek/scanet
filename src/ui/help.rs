use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_help(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![
            Span::styled("scanet", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" - LAN Device Scanner"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  j / ↓  ", Style::default().fg(Color::Green)),
            Span::raw("Move selection down"),
        ]),
        Line::from(vec![
            Span::styled("  k / ↑  ", Style::default().fg(Color::Green)),
            Span::raw("Move selection up"),
        ]),
        Line::from(vec![
            Span::styled("  Enter  ", Style::default().fg(Color::Green)),
            Span::raw("Show device details"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Scanning", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  s      ", Style::default().fg(Color::Green)),
            Span::raw("Start manual scan"),
        ]),
        Line::from(vec![
            Span::styled("  c      ", Style::default().fg(Color::Green)),
            Span::raw("Toggle continuous mode"),
        ]),
        Line::from(vec![
            Span::styled("  + / -  ", Style::default().fg(Color::Green)),
            Span::raw("Adjust scan interval"),
        ]),
        Line::from(vec![
            Span::styled("  r      ", Style::default().fg(Color::Green)),
            Span::raw("Set scan range"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ?      ", Style::default().fg(Color::Green)),
            Span::raw("Show this help"),
        ]),
        Line::from(vec![
            Span::styled("  q      ", Style::default().fg(Color::Green)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default().fg(Color::DarkGray))),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(help, area);
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
