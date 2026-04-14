pub mod commands;
pub mod dashboard;
pub mod history;
pub mod projects;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
};

use crate::app::{App, Tab};

/// Strip control characters (ESC, BEL, etc.) to prevent terminal escape injection.
/// Preserves printable chars and tabs only.
pub fn sanitize(s: &str) -> String {
    s.chars().filter(|&c| c >= ' ' || c == '\t').collect()
}

pub fn render(frame: &mut Frame, app: &App) {
    let has_status = app.last_error.is_some() || app.export_msg.is_some() || app.search_mode;
    let chunks = Layout::vertical([
        Constraint::Length(3),                              // tab bar
        Constraint::Min(0),                                 // content
        Constraint::Length(if has_status { 1 } else { 0 }), // status bar
    ])
    .split(frame.area());

    render_tabs(frame, app, chunks[0]);

    // Empty state: no data at all
    if app.cache.summary.total_commands == 0 {
        render_empty_state(frame, chunks[1]);
    } else {
        match app.tab {
            Tab::Dashboard => dashboard::render(frame, app, chunks[1]),
            Tab::History => history::render(frame, app, chunks[1]),
            Tab::Commands => commands::render(frame, app, chunks[1]),
            Tab::Projects => projects::render(frame, app, chunks[1]),
        }
    }

    // Status bar (priority: error > search > export)
    if let Some(err) = &app.last_error {
        let msg = format!(" Error: {} ", err);
        let paragraph = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::White).bg(Color::Red),
        )));
        frame.render_widget(paragraph, chunks[2]);
    } else if app.search_mode {
        let msg = format!(" / {}█ ", app.search_query);
        let paragraph = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::White).bg(Color::Blue),
        )));
        frame.render_widget(paragraph, chunks[2]);
    } else if let Some(msg) = &app.export_msg {
        let paragraph = Paragraph::new(Line::from(Span::styled(
            format!(" {msg} "),
            Style::default().fg(Color::White).bg(Color::Green),
        )));
        frame.render_widget(paragraph, chunks[2]);
    }

    // Help popup overlay (rendered last, on top of everything)
    if app.show_help {
        render_help_popup(frame);
    }
}

fn render_empty_state(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "No RTK data found.",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Run some commands with RTK to start tracking:",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  cargo install rtk",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "  rtk git status",
            Style::default().fg(Color::Cyan),
        )),
    ];
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn render_help_popup(frame: &mut Frame) {
    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  1 2 3 4  ", Style::default().fg(Color::Cyan)),
            Span::raw("Switch tabs"),
        ]),
        Line::from(vec![
            Span::styled("  Tab      ", Style::default().fg(Color::Cyan)),
            Span::raw("Next tab"),
        ]),
        Line::from(vec![
            Span::styled("  j / ↓    ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll down"),
        ]),
        Line::from(vec![
            Span::styled("  k / ↑    ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll up"),
        ]),
        Line::from(vec![
            Span::styled("  d w m    ", Style::default().fg(Color::Cyan)),
            Span::raw("Daily / Weekly / Monthly (History tab)"),
        ]),
        Line::from(vec![
            Span::styled("  /        ", Style::default().fg(Color::Cyan)),
            Span::raw("Search (Commands / Projects)"),
        ]),
        Line::from(vec![
            Span::styled("  e        ", Style::default().fg(Color::Cyan)),
            Span::raw("Export current tab as CSV"),
        ]),
        Line::from(vec![
            Span::styled("  r        ", Style::default().fg(Color::Cyan)),
            Span::raw("Force refresh"),
        ]),
        Line::from(vec![
            Span::styled("  q / Esc  ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let popup_width = 48;
    let popup_height = help_text.len() as u16 + 2; // +2 for border
    let area = frame.area();
    let popup_area = centered_rect(popup_width, popup_height, area);

    frame.render_widget(Clear, popup_area);
    let paragraph = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help (?) ")
            .border_style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(paragraph, popup_area);
}

/// Create a centered rectangle within `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);
    Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
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
        if i > 0 && (abs.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Format token counts with K/M suffix (matching RTK's `format_tokens`):
/// 1_234_567 → "1.2M", 59_234 → "59.2K", 694 → "694"
pub fn format_tokens(n: i64) -> String {
    let abs = n.unsigned_abs();
    let prefix = if n < 0 { "-" } else { "" };
    if abs >= 1_000_000 {
        format!("{prefix}{:.1}M", abs as f64 / 1_000_000.0)
    } else if abs >= 1_000 {
        format!("{prefix}{:.1}K", abs as f64 / 1_000.0)
    } else {
        format!("{prefix}{abs}")
    }
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

    #[test]
    fn test_format_tokens_millions() {
        assert_eq!(format_tokens(1_234_567), "1.2M");
        assert_eq!(format_tokens(12_345_678), "12.3M");
    }

    #[test]
    fn test_format_tokens_thousands() {
        assert_eq!(format_tokens(59_234), "59.2K");
        assert_eq!(format_tokens(1_000), "1.0K");
    }

    #[test]
    fn test_format_tokens_small() {
        assert_eq!(format_tokens(694), "694");
        assert_eq!(format_tokens(0), "0");
    }

    #[test]
    fn test_format_tokens_negative() {
        assert_eq!(format_tokens(-5_000), "-5.0K");
    }
}
