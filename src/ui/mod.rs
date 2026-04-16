pub mod commands;
pub mod dashboard;
pub mod history;
pub mod layout;
pub mod projects;
pub mod table_utils;
pub mod theme;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
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
    let search_hint_visible = matches!(app.tab, Tab::Commands | Tab::Projects);
    let has_status = app.last_error.is_some()
        || app.export_msg.is_some()
        || app.search_mode
        || search_hint_visible;
    let chunks = Layout::vertical([
        Constraint::Length(layout::TAB_BAR_HEIGHT),
        Constraint::Min(0),
        Constraint::Length(if has_status {
            layout::STATUS_BAR_HEIGHT
        } else {
            0
        }),
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

    if has_status {
        render_status_bar(frame, app, chunks[2], search_hint_visible);
    }

    // Help popup overlay (rendered last, on top of everything)
    if app.show_help {
        render_help_popup(frame);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect, search_hint_visible: bool) {
    let line = if let Some(err) = &app.last_error {
        active_status_line("ERROR", err, theme::ERROR_COLOR, area.width as usize)
    } else if app.search_mode {
        active_status_line(
            "SEARCH",
            &format!("/ {}█", app.search_query),
            theme::INFO_COLOR,
            area.width as usize,
        )
    } else if let Some(msg) = &app.export_msg {
        active_status_line("EXPORTED", msg, theme::SAVED_COLOR, area.width as usize)
    } else if search_hint_visible {
        passive_status_line(
            if app.search_query.is_empty() {
                "Tip · / search · ? help".to_string()
            } else {
                format!("Filter · {} · / edit", app.search_query)
            },
            area.width as usize,
        )
    } else {
        passive_status_line(
            "Tip · ? help · e export · r refresh".to_string(),
            area.width as usize,
        )
    };

    frame.render_widget(Paragraph::new(line), area);
}

fn active_status_line(
    label: &str,
    message: &str,
    background: ratatui::style::Color,
    width: usize,
) -> Line<'static> {
    let prefix = format!(" {label} ");
    let available = width.saturating_sub(prefix.chars().count() + 1);
    let message = truncate_inline(message, available);

    Line::from(vec![
        Span::styled(
            prefix,
            theme::status_style(background).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {message}"), theme::status_style(background)),
    ])
}

fn passive_status_line(message: String, width: usize) -> Line<'static> {
    let prefix = " Tip ";
    let available = width.saturating_sub(prefix.chars().count() + 1);
    let message = truncate_inline(&message, available);

    Line::from(vec![
        Span::styled(prefix, theme::bold(theme::HEADER_COLOR)),
        Span::styled(format!(" {message}"), theme::passive_status_style()),
    ])
}

fn truncate_inline(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        text.to_string()
    } else if max_chars == 1 {
        "…".to_string()
    } else {
        let mut truncated: String = chars[..max_chars - 1].iter().collect();
        truncated.push('…');
        truncated
    }
}

fn render_empty_state(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "No RTK data found.",
            theme::bold(theme::HEADER_COLOR),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Run some commands with RTK to start tracking:",
            Style::default().fg(theme::MUTED_COLOR),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  cargo install rtk",
            Style::default().fg(theme::PERCENTAGE_COLOR),
        )),
        Line::from(Span::styled(
            "  rtk git status",
            Style::default().fg(theme::PERCENTAGE_COLOR),
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
            theme::bold(theme::HEADER_COLOR),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  1 2 3 4  ", Style::default().fg(theme::PERCENTAGE_COLOR)),
            Span::raw("Switch tabs"),
        ]),
        Line::from(vec![
            Span::styled("  Tab      ", Style::default().fg(theme::PERCENTAGE_COLOR)),
            Span::raw("Next tab"),
        ]),
        Line::from(vec![
            Span::styled("  j / ↓    ", Style::default().fg(theme::PERCENTAGE_COLOR)),
            Span::raw("Scroll down"),
        ]),
        Line::from(vec![
            Span::styled("  k / ↑    ", Style::default().fg(theme::PERCENTAGE_COLOR)),
            Span::raw("Scroll up"),
        ]),
        Line::from(vec![
            Span::styled("  d w m    ", Style::default().fg(theme::PERCENTAGE_COLOR)),
            Span::raw("Daily / Weekly / Monthly (History tab)"),
        ]),
        Line::from(vec![
            Span::styled("  /        ", Style::default().fg(theme::PERCENTAGE_COLOR)),
            Span::raw("Search (Commands / Projects)"),
        ]),
        Line::from(vec![
            Span::styled("  e        ", Style::default().fg(theme::PERCENTAGE_COLOR)),
            Span::raw("Export current tab as CSV"),
        ]),
        Line::from(vec![
            Span::styled("  r        ", Style::default().fg(theme::PERCENTAGE_COLOR)),
            Span::raw("Force refresh"),
        ]),
        Line::from(vec![
            Span::styled("  q / Esc  ", Style::default().fg(theme::PERCENTAGE_COLOR)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(theme::MUTED_COLOR),
        )),
    ];

    let popup_width = layout::HELP_POPUP_WIDTH;
    let popup_height = help_text.len() as u16 + 2; // +2 for border
    let area = frame.area();
    let popup_area = centered_rect(popup_width, popup_height, area);

    frame.render_widget(Clear, popup_area);
    let paragraph = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help (?) ")
            .border_style(Style::default().fg(theme::HEADER_COLOR)),
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
                Span::styled(num, Style::default().fg(theme::MUTED_COLOR)),
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
        .highlight_style(theme::tab_highlight_style());

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
