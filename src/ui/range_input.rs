use crate::app::{App, RangeInputField};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_range_input(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 40, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Start label
            Constraint::Length(3), // Start input (with borders)
            Constraint::Length(1), // End label
            Constraint::Length(3), // End input (with borders)
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Error message
            Constraint::Min(1),    // Help text
        ])
        .split(area);

    // Background block
    let block = Block::default()
        .title(" Scan Range ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    // Title
    let title = Paragraph::new("CIDR (192.168.1.0/24) or Start-End IP")
        .style(Style::default().fg(Color::White));
    frame.render_widget(title, chunks[0]);

    // Start IP field
    let start_style = if app.range_input_field == RangeInputField::Start {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let start_label = Paragraph::new("Start/CIDR:").style(start_style);
    frame.render_widget(start_label, chunks[2]);

    let start_cursor = if app.range_input_field == RangeInputField::Start {
        Some(app.range_input_cursor)
    } else {
        None
    };
    let start_input = render_input_field(
        &app.range_input_start,
        app.range_input_field == RangeInputField::Start,
        start_cursor,
    );
    frame.render_widget(start_input, chunks[3]);

    // End IP field
    let end_style = if app.range_input_field == RangeInputField::End {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let end_label = Paragraph::new("End IP:").style(end_style);
    frame.render_widget(end_label, chunks[4]);

    let end_cursor = if app.range_input_field == RangeInputField::End {
        Some(app.range_input_cursor)
    } else {
        None
    };
    let end_input = render_input_field(
        &app.range_input_end,
        app.range_input_field == RangeInputField::End,
        end_cursor,
    );
    frame.render_widget(end_input, chunks[5]);

    // Error message
    if let Some(error) = &app.range_input_error {
        let error_text = Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
        frame.render_widget(error_text, chunks[7]);
    }

    // Help text
    let help_text = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Green)),
        Span::raw(" Switch field  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" Apply  "),
        Span::styled("Esc", Style::default().fg(Color::Green)),
        Span::raw(" Cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help_text, chunks[8]);
}

fn render_input_field(
    value: &str,
    is_focused: bool,
    cursor_pos: Option<usize>,
) -> Paragraph<'static> {
    let style = if is_focused {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let display_value = if let Some(pos) = cursor_pos {
        let pos = pos.min(value.len());
        let (before, after) = value.split_at(pos);
        format!("{}|{}", before, after)
    } else {
        value.to_string()
    };

    Paragraph::new(display_value)
        .style(style)
        .block(Block::default().borders(Borders::ALL))
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
