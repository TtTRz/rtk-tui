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
        Constraint::Length(7),  // last 24h sparkline + axis
        Constraint::Length(7),  // last 30 days sparkline + axis
        Constraint::Min(0),     // recent commands
    ])
    .split(area);

    render_summary(frame, app, chunks[0]);
    render_last_24h(frame, app, chunks[1]);
    render_sparkline(frame, app, chunks[2]);
    render_recent(frame, app, chunks[3]);
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

/// Stretch data points to fill the target width using nearest-neighbor sampling.
/// Sparkline renders 1 bar per data point, so we need data.len() == display width.
fn stretch_data(data: &[u64], target_width: usize) -> Vec<u64> {
    if data.is_empty() || target_width == 0 {
        return vec![0; target_width];
    }
    let n = data.len();
    (0..target_width)
        .map(|i| {
            let src = i * n / target_width;
            data[src.min(n - 1)]
        })
        .collect()
}

fn render_last_24h(frame: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " Last 24 Hours — Tokens Saved ({}) ",
        format_tokens(app.cache.saved_last_24h)
    );

    let inner = Layout::vertical([
        Constraint::Min(1),    // sparkline with top+left+right border
        Constraint::Length(1), // axis labels (no border, just text)
        Constraint::Length(1), // bottom border line
    ])
    .split(area);

    // Sparkline inner width = area width - 2 (left+right border)
    let spark_width = inner[0].width.saturating_sub(2) as usize;
    let stretched = stretch_data(&app.cache.sparkline_24h, spark_width);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .title(title),
        )
        .data(&stretched)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(sparkline, inner[0]);

    // Axis labels: plain text padded to align inside the border columns
    let axis = build_hour_axis(inner[1].width as usize);
    let paragraph = Paragraph::new(axis);
    frame.render_widget(paragraph, inner[1]);

    // Close bottom border
    let bottom = Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT);
    frame.render_widget(bottom, inner[2]);
}

/// Build hour axis label line: "-24h    -18h    -12h    -6h     now"
fn build_hour_axis(width: usize) -> Line<'static> {
    let inner_w = width.saturating_sub(2);
    if inner_w < 10 {
        return Line::from("");
    }
    let labels: Vec<String> = vec![
        "-24h".to_string(),
        "-18h".to_string(),
        "-12h".to_string(),
        "-6h".to_string(),
        "now".to_string(),
    ];
    wrap_axis_in_border(&labels, inner_w)
}

fn render_sparkline(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Layout::vertical([
        Constraint::Min(1),    // sparkline with top+left+right border
        Constraint::Length(1), // axis labels
        Constraint::Length(1), // bottom border line
    ])
    .split(area);

    let spark_width = inner[0].width.saturating_sub(2) as usize;
    let stretched = stretch_data(&app.cache.sparkline, spark_width);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .title(" Last 30 Days — Tokens Saved "),
        )
        .data(&stretched)
        .style(Style::default().fg(Color::Green));
    frame.render_widget(sparkline, inner[0]);

    let axis = build_day_axis(inner[1].width as usize);
    let paragraph = Paragraph::new(axis);
    frame.render_widget(paragraph, inner[1]);

    let bottom = Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT);
    frame.render_widget(bottom, inner[2]);
}

/// Build day axis label line with dates spread across width.
fn build_day_axis(width: usize) -> Line<'static> {
    let inner_w = width.saturating_sub(2);
    if inner_w < 10 {
        return Line::from("");
    }
    let today = chrono::Local::now().date_naive();
    let offsets = [29, 22, 14, 7, 0];
    let labels: Vec<String> = offsets
        .iter()
        .map(|&days_ago| {
            let d = today - chrono::Duration::days(days_ago);
            d.format("%m/%d").to_string()
        })
        .collect();
    wrap_axis_in_border(&labels, inner_w)
}

/// Render axis labels between "│" border chars: "│ label1    label2    label3 │"
fn wrap_axis_in_border(labels: &[String], inner_w: usize) -> Line<'static> {
    if labels.is_empty() || inner_w == 0 {
        return Line::from("");
    }
    let style = Style::default().fg(Color::DarkGray);
    let border_style = Style::default().fg(Color::White);

    let total_label_len: usize = labels.iter().map(|l| l.len()).sum();
    if total_label_len >= inner_w {
        let text: String = labels.join(" ");
        return Line::from(vec![
            Span::styled("│", border_style),
            Span::styled(text, style),
            Span::styled("│", border_style),
        ]);
    }

    let n = labels.len();
    let total_gap = inner_w - total_label_len;
    let gap_count = if n > 1 { n - 1 } else { 1 };

    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled("│", border_style));
    for (i, label) in labels.iter().enumerate() {
        spans.push(Span::styled(label.clone(), style));
        if i < n - 1 {
            let gap = total_gap * (i + 1) / gap_count - total_gap * i / gap_count;
            spans.push(Span::raw(" ".repeat(gap)));
        }
    }
    spans.push(Span::styled("│", border_style));
    Line::from(spans)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_cmd_short() {
        assert_eq!(truncate_cmd("git status", 30), "git status");
    }

    #[test]
    fn test_truncate_cmd_exact() {
        let s = "a".repeat(30);
        assert_eq!(truncate_cmd(&s, 30), s);
    }

    #[test]
    fn test_truncate_cmd_long() {
        let s = "a".repeat(40);
        let result = truncate_cmd(&s, 30);
        assert_eq!(result.chars().count(), 30);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn test_truncate_cmd_unicode() {
        let s = "日本語コマンド名前がとても長い場合のテスト用文字列です";
        let result = truncate_cmd(s, 10);
        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn test_format_duration_millis() {
        assert_eq!(format_duration(42), "42ms");
        assert_eq!(format_duration(0), "0ms");
        assert_eq!(format_duration(999), "999ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(59_999), "60.0s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60_000), "1m0s");
        assert_eq!(format_duration(562_000), "9m22s");
    }

    #[test]
    fn test_format_duration_negative() {
        assert_eq!(format_duration(-1500), "1.5s");
    }
}
