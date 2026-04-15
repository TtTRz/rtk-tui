use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::{format_number, format_tokens, sanitize};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let h = area.height;
    // Adaptive layout based on terminal height
    if h >= 30 {
        // Tall terminal: show all sections
        let chunks = Layout::vertical([
            Constraint::Length(13), // summary + buddy
            Constraint::Length(8),  // last 24h bar chart + axis
            Constraint::Length(8),  // last 30 days bar chart + axis
            Constraint::Min(0),     // recent commands
        ])
        .split(area);
        render_summary(frame, app, chunks[0]);
        render_last_24h(frame, app, chunks[1]);
        render_sparkline(frame, app, chunks[2]);
        render_recent(frame, app, chunks[3]);
    } else if h >= 20 {
        // Medium terminal: smaller sparklines
        let chunks = Layout::vertical([
            Constraint::Length(11), // summary + buddy
            Constraint::Length(6),  // last 24h bar chart
            Constraint::Length(6),  // last 30 days bar chart
            Constraint::Min(0),     // recent commands
        ])
        .split(area);
        render_summary(frame, app, chunks[0]);
        render_last_24h(frame, app, chunks[1]);
        render_sparkline(frame, app, chunks[2]);
        render_recent(frame, app, chunks[3]);
    } else {
        // Short terminal: skip sparklines
        let chunks = Layout::vertical([
            Constraint::Length(11), // summary + buddy
            Constraint::Min(0),     // recent commands
        ])
        .split(area);
        render_summary(frame, app, chunks[0]);
        render_recent(frame, app, chunks[1]);
    }
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
    // Only show buddy if terminal is wide enough (>= 75 cols)
    if area.width >= 75 {
        let cols = Layout::horizontal([
            Constraint::Percentage(60), // KPI
            Constraint::Percentage(40), // Buddy
        ])
        .split(area);

        render_kpi(frame, app, cols[0]);
        crate::buddy::render_buddy(frame, app, cols[1]);
    } else {
        render_kpi(frame, app, area);
    }
}

/// Update buddy's max_x based on actual panel width. Called from App on tick.
pub fn update_buddy_max_x(app: &mut crate::app::App, area_width: u16) {
    if area_width >= 75 {
        let buddy_w = (area_width as usize * 40 / 100).saturating_sub(2); // 40% minus border
        app.buddy.set_max_x(buddy_w);
    }
}

fn render_kpi(frame: &mut Frame, app: &App, area: Rect) {
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

/// Bar characters for 8 height levels (index 0 = lowest, 7 = tallest).
const BAR_CHARS: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

/// Map a data value to a color based on its percentile of the max.
/// Peak → muted red, High → muted amber, Medium → base_color, Low → DarkGray
fn value_to_color(val: u64, max_val: u64, base_color: Color) -> Color {
    if val == 0 || max_val == 0 {
        return Color::DarkGray;
    }
    let ratio = val as f64 / max_val as f64;
    if ratio >= 0.9 {
        Color::Rgb(200, 100, 100) // muted red
    } else if ratio >= 0.6 {
        Color::Rgb(200, 170, 80) // muted amber
    } else if ratio >= 0.2 {
        base_color
    } else {
        Color::DarkGray
    }
}

/// Build multi-line colored bar chart. Bars grow upward from bottom (adjacent to axis).
/// `content_height` = number of text lines for bars.
/// Total resolution = content_height * 8 levels.
fn build_bar_lines(data: &[u64], base_color: Color, content_height: usize) -> Vec<Line<'static>> {
    let max_val = data.iter().copied().max().unwrap_or(0);
    let total_levels = content_height * 8;

    // For each data point, compute its level (0..total_levels)
    let levels: Vec<usize> = data
        .iter()
        .map(|&v| {
            if v == 0 || max_val == 0 {
                0
            } else {
                // Non-zero gets at least level 1
                let l = ((v as f64 / max_val as f64) * total_levels as f64).round() as usize;
                l.clamp(1, total_levels)
            }
        })
        .collect();

    // Render top-to-bottom: row 0 = top, row (content_height-1) = bottom (touching axis)
    (0..content_height)
        .map(|row| {
            // This row covers levels: base_level..base_level+8
            // row 0 (top) = highest levels, row (content_height-1) (bottom) = levels 0..8
            let base_level = (content_height - 1 - row) * 8;
            let spans: Vec<Span<'static>> = levels
                .iter()
                .zip(data.iter())
                .map(|(&level, &val)| {
                    let color = value_to_color(val, max_val, base_color);
                    if level <= base_level {
                        // Bar doesn't reach this row
                        Span::styled(" ".to_string(), Style::default())
                    } else if level >= base_level + 8 {
                        // Bar fills this row completely
                        Span::styled("█".to_string(), Style::default().fg(color))
                    } else {
                        // Partial fill
                        let sub = level - base_level; // 1..7
                        Span::styled(
                            BAR_CHARS[sub.min(7)].to_string(),
                            Style::default().fg(color),
                        )
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

/// Stretch data points to fill the target width using nearest-neighbor sampling.
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
        Constraint::Length(6), // top border + 5 bar lines
        Constraint::Length(1), // axis labels
        Constraint::Length(1), // bottom border line
    ])
    .split(area);

    // Inner width = area width - 2 (left+right border)
    let spark_width = inner[0].width.saturating_sub(2) as usize;
    let stretched = stretch_data(&app.cache.sparkline_24h, spark_width);
    // content_height = area height - 1 (top border)
    let content_h = inner[0].height.saturating_sub(1) as usize;
    let bar_lines = build_bar_lines(&stretched, Color::Cyan, content_h);

    let paragraph = Paragraph::new(bar_lines).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(title),
    );
    frame.render_widget(paragraph, inner[0]);

    // Axis labels: plain text padded to align inside the border columns
    let axis = build_hour_axis(inner[1].width as usize);
    let paragraph = Paragraph::new(axis);
    frame.render_widget(paragraph, inner[1]);

    // Close bottom border
    let bottom = Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT);
    frame.render_widget(bottom, inner[2]);
}

/// Build hour axis label line with actual clock times: "17:00  20:00  23:00  02:00  05:00"
fn build_hour_axis(width: usize) -> Line<'static> {
    let inner_w = width.saturating_sub(2);
    if inner_w < 10 {
        return Line::from("");
    }
    let now = chrono::Local::now();
    // Adaptive label count based on width: every 2h if wide enough, else every 3h
    let step = if inner_w >= 60 { 2 } else { 3 };
    let count = 24 / step + 1;
    let labels: Vec<String> = (0..count)
        .map(|i| {
            let hours_ago = 24 - i * step;
            if hours_ago == 0 {
                "now".to_string()
            } else {
                let t = now - chrono::Duration::hours(hours_ago as i64);
                t.format("%H:%M").to_string()
            }
        })
        .collect();
    wrap_axis_in_border(&labels, inner_w)
}

fn render_sparkline(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Layout::vertical([
        Constraint::Length(6), // top border + 5 bar lines
        Constraint::Length(1), // axis labels
        Constraint::Length(1), // bottom border line
    ])
    .split(area);

    let spark_width = inner[0].width.saturating_sub(2) as usize;
    let stretched = stretch_data(&app.cache.sparkline, spark_width);
    let content_h = inner[0].height.saturating_sub(1) as usize;
    let bar_lines = build_bar_lines(&stretched, Color::Green, content_h);

    let paragraph = Paragraph::new(bar_lines).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(" Last 30 Days — Tokens Saved "),
    );
    frame.render_widget(paragraph, inner[0]);

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
    // Adaptive label count: every 3 days if wide, else every 5 days
    let step = if inner_w >= 60 { 3 } else { 5 };
    let count = 30 / step + 1;
    let labels: Vec<String> = (0..count)
        .map(|i| {
            let days_ago = 30 - i * step;
            if days_ago == 0 {
                "now".to_string()
            } else {
                let d = today - chrono::Duration::days(days_ago as i64);
                d.format("%m/%d").to_string()
            }
        })
        .collect();
    wrap_axis_in_border(&labels, inner_w)
}

/// Render axis labels between "│" border chars, with all labels padded to uniform width.
fn wrap_axis_in_border(labels: &[String], inner_w: usize) -> Line<'static> {
    if labels.is_empty() || inner_w == 0 {
        return Line::from("");
    }
    let style = Style::default().fg(Color::DarkGray);
    let border_style = Style::default().fg(Color::White);

    // Pad all labels to the same width (center-aligned)
    let max_label_len = labels.iter().map(|l| l.len()).max().unwrap_or(0);
    let padded: Vec<String> = labels
        .iter()
        .map(|l| format!("{:^width$}", l, width = max_label_len))
        .collect();

    let total_label_len: usize = padded.iter().map(|l| l.len()).sum();
    if total_label_len >= inner_w {
        let text: String = padded.join(" ");
        return Line::from(vec![
            Span::styled("│", border_style),
            Span::styled(text, style),
            Span::styled("│", border_style),
        ]);
    }

    let n = padded.len();
    let total_gap = inner_w - total_label_len;
    let gap_count = if n > 1 { n - 1 } else { 1 };

    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled("│", border_style));
    for (i, label) in padded.iter().enumerate() {
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

#[cfg(test)]
mod render_tests {
    use crate::app::App;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn make_test_app() -> App {
        // Create a minimal app with fake data (no DB needed for rendering)
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE commands (
                id INTEGER PRIMARY KEY, timestamp TEXT NOT NULL,
                original_cmd TEXT NOT NULL, rtk_cmd TEXT NOT NULL,
                input_tokens INTEGER NOT NULL, output_tokens INTEGER NOT NULL,
                saved_tokens INTEGER NOT NULL, savings_pct REAL NOT NULL,
                exec_time_ms INTEGER DEFAULT 0, project_path TEXT DEFAULT ''
            );
            INSERT INTO commands VALUES (1, datetime('now'), 'git status', 'rtk git status', 1000, 200, 800, 80.0, 5, '');",
        ).unwrap();
        drop(conn);
        let db = crate::db::Db::open(Some(db_path.to_str().unwrap())).unwrap();
        App::new(db, 1, db_path.to_str().unwrap(), None)
    }

    #[test]
    fn test_buddy_renders_in_dashboard() {
        let app = make_test_app();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                crate::ui::render(frame, &app);
            })
            .unwrap();

        let buf = terminal.backend().buffer().clone();
        // Search for buddy-related content in the buffer
        let content: String = (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Dump full buffer to file for debugging
        let mut dump = String::new();
        dump.push_str("=== FULL RENDER OUTPUT (80x24) ===\n");
        for y in 0..buf.area.height {
            let line: String = (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect();
            dump.push_str(&format!("{:2}: {}\n", y, line));
        }
        dump.push_str("=== END ===\n");
        std::fs::write("/tmp/rtk-tui-render-dump.txt", &dump).unwrap();

        assert!(
            content.contains("Buddy"),
            "Buddy block title not found in render output"
        );
    }
}
