use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use super::{format_number, sanitize};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(5), // summary cards
        Constraint::Length(8), // sparkline
        Constraint::Min(0),    // recent commands
    ])
    .split(area);

    render_summary(frame, app, chunks[0]);
    render_sparkline(frame, app, chunks[1]);
    render_recent(frame, app, chunks[2]);
}

fn make_card<'a>(title: &'a str, value: &'a str, color: Color) -> Paragraph<'a> {
    Paragraph::new(vec![
        Line::from(Span::styled(title, Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
    ])
    .block(Block::default().borders(Borders::ALL))
}

fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    let summary = &app.cache.summary;

    let cols = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .split(area);

    let saved = format_number(summary.total_saved);
    let avg = format!("{:.1}%", summary.avg_savings_pct);
    let cmds = format_number(summary.total_commands);
    let time = format!("{:.1}s", summary.total_time_ms as f64 / 1000.0);

    frame.render_widget(make_card("Tokens Saved", &saved, Color::Green), cols[0]);
    frame.render_widget(make_card("Avg Savings", &avg, Color::Cyan), cols[1]);
    frame.render_widget(make_card("Commands", &cmds, Color::Yellow), cols[2]);
    frame.render_widget(make_card("Total Time", &time, Color::Magenta), cols[3]);
}

fn render_sparkline(frame: &mut Frame, app: &App, area: Rect) {
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Last 30 Days — Tokens Saved "),
        )
        .data(&app.cache.sparkline)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(sparkline, area);
}

fn render_recent(frame: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app
        .cache
        .recent
        .iter()
        .map(|r| {
            // UTF-8 safe timestamp truncation
            let ts: String = r.timestamp.chars().take(19).collect();
            Line::from(vec![
                Span::styled(format!("{ts:<20}"), Style::default().fg(Color::DarkGray)),
                Span::raw(format!(" {:<30}", sanitize(&r.rtk_cmd))),
                Span::styled(
                    format!(" {:>+6}", r.saved_tokens),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!(" ({:.0}%)", r.savings_pct),
                    Style::default().fg(Color::Cyan),
                ),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Recent Commands "),
    );

    frame.render_widget(paragraph, area);
}
