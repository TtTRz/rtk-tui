use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
};

use super::{format_number, format_tokens, sanitize};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(11), // summary + efficiency meter
        Constraint::Length(8),  // sparkline
        Constraint::Min(0),     // recent commands
    ])
    .split(area);

    render_summary(frame, app, chunks[0]);
    render_sparkline(frame, app, chunks[1]);
    render_recent(frame, app, chunks[2]);
}

/// Format milliseconds to human-readable duration (matching RTK's format_duration).
fn format_duration(ms: i64) -> String {
    let ms = ms.unsigned_abs();
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1000;
        format!("{minutes}m{seconds}s")
    }
}

/// Truncate a command string to `max` characters, appending `…` if truncated.
fn truncate_cmd(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let mut t: String = chars[..max.saturating_sub(1)].iter().collect();
        t.push('…');
        t
    }
}

/// Build efficiency meter bar: ██████████████████░░░░░░ 93.0%
fn efficiency_meter(pct: f64) -> Line<'static> {
    let width = 24usize;
    let ratio = (pct / 100.0).clamp(0.0, 1.0);
    let filled = ((ratio * width as f64).round() as usize).min(width);

    let color = if pct >= 70.0 {
        Color::Green
    } else if pct >= 40.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let bar_filled = "█".repeat(filled);
    let bar_empty = "░".repeat(width - filled);
    let label = format!(" {pct:.1}%");

    Line::from(vec![
        Span::styled("Efficiency:       ", Style::default().fg(Color::DarkGray)),
        Span::styled(bar_filled, Style::default().fg(color)),
        Span::styled(bar_empty, Style::default().fg(Color::DarkGray)),
        Span::styled(
            label,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.cache.summary;

    let lines = vec![
        make_kpi_line(
            "Total commands:   ",
            &format_number(s.total_commands),
            Color::White,
        ),
        make_kpi_line(
            "Input tokens:     ",
            &format_tokens(s.total_input),
            Color::White,
        ),
        make_kpi_line(
            "Output tokens:    ",
            &format_tokens(s.total_output),
            Color::White,
        ),
        make_kpi_line(
            "Tokens saved:     ",
            &format!(
                "{} ({:.1}%)",
                format_tokens(s.total_saved),
                s.avg_savings_pct
            ),
            Color::Green,
        ),
        make_kpi_line(
            "Total exec time:  ",
            &format!(
                "{} (avg {})",
                format_duration(s.total_time_ms),
                format_duration(s.avg_time_ms)
            ),
            Color::Magenta,
        ),
        Line::from(""),
        efficiency_meter(s.avg_savings_pct),
    ];

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Summary "));
    frame.render_widget(paragraph, area);
}

fn make_kpi_line<'a>(label: &'a str, value: &str, color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::DarkGray)),
        Span::styled(
            value.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
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
            let ts: String = r.timestamp.chars().take(19).collect();
            Line::from(vec![
                Span::styled(format!("{ts:<20}"), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {:<30}", truncate_cmd(&sanitize(&r.rtk_cmd), 30)),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("{:>8}", format_tokens(r.saved_tokens)),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!("{:>7}", format!("({:.0}%)", r.savings_pct)),
                    Style::default().fg(Color::DarkGray),
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
