pub mod commands;
pub mod dashboard;
pub mod history;
pub mod projects;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Tab};

/// Strip control characters (ESC, BEL, etc.) to prevent terminal escape injection.
/// Preserves printable chars and tabs only.
pub fn sanitize(s: &str) -> String {
    s.chars().filter(|&c| c >= ' ' || c == '\t').collect()
}

pub fn render(frame: &mut Frame, app: &App) {
    let has_error = app.last_error.is_some();
    let chunks = Layout::vertical([
        Constraint::Length(3),                             // tab bar
        Constraint::Min(0),                                // content
        Constraint::Length(if has_error { 1 } else { 0 }), // status bar
    ])
    .split(frame.area());

    render_tabs(frame, app, chunks[0]);

    match app.tab {
        Tab::Dashboard => dashboard::render(frame, app, chunks[1]),
        Tab::History => history::render(frame, app, chunks[1]),
        Tab::Commands => commands::render(frame, app, chunks[1]),
        Tab::Projects => projects::render(frame, app, chunks[1]),
    }

    if let Some(err) = &app.last_error {
        let msg = format!(" DB error: {} ", err);
        let paragraph = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::White).bg(Color::Red),
        )));
        frame.render_widget(paragraph, chunks[2]);
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let num = format!(" {} ", i + 1);
            Line::from(vec![
                Span::styled(num, Style::default().fg(Color::DarkGray)),
                Span::raw(t.title()),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" RTK Token Savings "),
        )
        .select(app.tab.index())
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

/// Format large numbers with commas: 1234567 → "1,234,567"
/// Handles negative values correctly.
pub fn format_number(n: i64) -> String {
    let neg = n < 0;
    let abs = n.unsigned_abs().to_string();
    let mut result = String::with_capacity(abs.len() + abs.len() / 3 + 1);
    if neg {
        result.push('-');
    }
    for (i, c) in abs.chars().enumerate() {
        if i > 0 && (abs.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Shorten a file path for display, UTF-8 safe.
/// /Users/foo/long/path/project → .../project
pub fn shorten_path(path: &str, max_len: usize) -> String {
    if path.chars().count() <= max_len {
        return path.to_string();
    }
    let parts: Vec<&str> = path.split(std::path::MAIN_SEPARATOR).collect();
    if let Some(last) = parts.last() {
        let short = format!("...{}{}", std::path::MAIN_SEPARATOR, last);
        if short.chars().count() <= max_len {
            return short;
        }
    }
    // UTF-8 safe: take the last (max_len - 3) characters
    let chars: Vec<char> = path.chars().collect();
    let take_from = chars.len().saturating_sub(max_len.saturating_sub(3));
    let truncated: String = chars[take_from..].iter().collect();
    format!("...{truncated}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_positive() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1_234_567), "1,234,567");
    }

    #[test]
    fn test_format_number_negative() {
        assert_eq!(format_number(-1), "-1");
        assert_eq!(format_number(-1000), "-1,000");
    }

    #[test]
    fn test_shorten_path_short() {
        assert_eq!(shorten_path("/a/b", 20), "/a/b");
    }

    #[test]
    fn test_shorten_path_long() {
        let long = "/Users/romchung/very/long/path/to/project";
        let short = shorten_path(long, 20);
        assert!(short.starts_with("..."));
        assert!(short.chars().count() <= 20);
    }

    #[test]
    fn test_shorten_path_unicode() {
        let path = "/home/用户/项目/测试目录";
        let short = shorten_path(path, 15);
        assert!(short.starts_with("..."));
        // Should not panic on multibyte chars
    }

    #[test]
    fn test_sanitize_strips_escape_sequences() {
        assert_eq!(sanitize("normal text"), "normal text");
        assert_eq!(sanitize("has\x1b[31mred\x1b[0m"), "has[31mred[0m");
        assert_eq!(sanitize("bell\x07here"), "bellhere");
        assert_eq!(sanitize("tab\there"), "tab\there"); // tabs preserved
    }
}
