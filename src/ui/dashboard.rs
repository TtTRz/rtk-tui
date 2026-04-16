use std::borrow::Cow;

use chrono::Timelike;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::{format_number, format_tokens, layout, sanitize, theme};
use crate::{app::App, db};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let h = area.height;
    let side_by_side_charts = area.width >= layout::DASHBOARD_CHARTS_SIDE_BY_SIDE_MIN_WIDTH;

    if h >= layout::DASHBOARD_TALL_MIN_HEIGHT {
        if side_by_side_charts {
            let chunks = Layout::vertical([
                Constraint::Length(layout::DASHBOARD_TALL_SUMMARY_HEIGHT),
                Constraint::Length(layout::DASHBOARD_TALL_CHART_HEIGHT),
                Constraint::Min(0),
            ])
            .split(area);
            render_summary(frame, app, chunks[0]);
            render_chart_row(frame, app, chunks[1]);
            render_recent(frame, app, chunks[2]);
        } else {
            let chunks = Layout::vertical([
                Constraint::Length(layout::DASHBOARD_TALL_SUMMARY_HEIGHT),
                Constraint::Length(layout::DASHBOARD_TALL_CHART_HEIGHT),
                Constraint::Length(layout::DASHBOARD_TALL_CHART_HEIGHT),
                Constraint::Min(0),
            ])
            .split(area);
            render_summary(frame, app, chunks[0]);
            render_last_24h(frame, app, chunks[1]);
            render_sparkline(frame, app, chunks[2]);
            render_recent(frame, app, chunks[3]);
        }
    } else if h >= layout::DASHBOARD_MEDIUM_MIN_HEIGHT {
        if side_by_side_charts {
            let chunks = Layout::vertical([
                Constraint::Length(layout::DASHBOARD_MEDIUM_SUMMARY_HEIGHT),
                Constraint::Length(layout::DASHBOARD_MEDIUM_CHART_HEIGHT),
                Constraint::Min(0),
            ])
            .split(area);
            render_summary(frame, app, chunks[0]);
            render_chart_row(frame, app, chunks[1]);
            render_recent(frame, app, chunks[2]);
        } else {
            let chunks = Layout::vertical([
                Constraint::Length(layout::DASHBOARD_MEDIUM_SUMMARY_HEIGHT),
                Constraint::Length(layout::DASHBOARD_MEDIUM_CHART_HEIGHT),
                Constraint::Length(layout::DASHBOARD_MEDIUM_CHART_HEIGHT),
                Constraint::Min(0),
            ])
            .split(area);
            render_summary(frame, app, chunks[0]);
            render_last_24h(frame, app, chunks[1]);
            render_sparkline(frame, app, chunks[2]);
            render_recent(frame, app, chunks[3]);
        }
    } else {
        let chunks = Layout::vertical([
            Constraint::Length(layout::DASHBOARD_MEDIUM_SUMMARY_HEIGHT),
            Constraint::Min(0),
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

const SUMMARY_LABEL_WIDTH: usize = 12;

/// Build efficiency meter bar: ██████████████████░░░░░░ 93.0%
fn efficiency_meter(pct: f64) -> Line<'static> {
    let width = 24usize;
    let ratio = (pct / 100.0).clamp(0.0, 1.0);
    let filled = ((ratio * width as f64).round() as usize).min(width);

    let color = if pct >= 70.0 {
        theme::SAVED_COLOR
    } else if pct >= 40.0 {
        theme::HEADER_COLOR
    } else {
        theme::ERROR_COLOR
    };

    let bar_filled = "█".repeat(filled);
    let bar_empty = "░".repeat(width - filled);
    let label = format!(" {pct:.1}%");

    Line::from(vec![
        summary_label_span("Efficiency:"),
        Span::styled(bar_filled, Style::default().fg(color)),
        Span::styled(bar_empty, Style::default().fg(theme::MUTED_COLOR)),
        Span::styled(label, theme::bold(color)),
    ])
}

fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    if area.width >= layout::BUDDY_MIN_WIDTH {
        let cols = Layout::horizontal([
            Constraint::Percentage(layout::KPI_WIDTH_PERCENT),
            Constraint::Percentage(layout::BUDDY_WIDTH_PERCENT),
        ])
        .split(area);

        render_kpi(frame, app, cols[0]);
        crate::buddy::render_buddy(frame, app, cols[1]);
    } else {
        render_kpi(frame, app, area);
    }
}

fn render_chart_row(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Ratio(1, 2),
        Constraint::Length(1),
        Constraint::Ratio(1, 2),
    ])
    .split(area);

    render_last_24h(frame, app, cols[0]);
    render_sparkline(frame, app, cols[2]);
}

/// Update buddy's max_x based on actual panel width. Called from App on tick.
pub fn update_buddy_max_x(app: &mut crate::app::App, area_width: u16) {
    if area_width >= layout::BUDDY_MIN_WIDTH {
        let buddy_w =
            (area_width as usize * layout::BUDDY_WIDTH_PERCENT as usize / 100).saturating_sub(2);
        app.buddy.set_max_x(buddy_w);
    }
}

fn render_kpi(frame: &mut Frame, app: &App, area: Rect) {
    let cards = Layout::vertical([Constraint::Length(5), Constraint::Min(7)]).split(area);

    render_summary_overview_card(frame, app, cards[0]);
    render_summary_details_card(frame, app, cards[1]);
}

fn render_summary_overview_card(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.cache.summary;
    let lines = vec![
        primary_saved_line(s),
        efficiency_meter(s.avg_savings_pct),
        trend_summary_line(app),
    ];

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Overview "));
    frame.render_widget(paragraph, area);
}

fn render_summary_details_card(frame: &mut Frame, app: &App, area: Rect) {
    let lines = summary_detail_lines(&app.cache.summary);
    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Details "));
    frame.render_widget(paragraph, area);
}

fn primary_saved_line(summary: &db::Summary) -> Line<'static> {
    let saved = format_tokens(summary.total_saved);

    Line::from(vec![
        summary_label_span("Saved:"),
        Span::styled(saved, theme::bold(theme::SAVED_COLOR)),
    ])
}

fn summary_detail_lines(summary: &db::Summary) -> Vec<Line<'static>> {
    vec![
        make_kpi_line(
            "Commands:",
            &format_number(summary.total_commands),
            theme::TEXT_COLOR,
        ),
        make_kpi_line(
            "Input:",
            &format_tokens(summary.total_input),
            theme::TEXT_COLOR,
        ),
        make_kpi_line(
            "Output:",
            &format_tokens(summary.total_output),
            theme::TEXT_COLOR,
        ),
        make_kpi_line(
            "Total time:",
            &format_duration(summary.total_time_ms),
            theme::TOTAL_TIME_COLOR,
        ),
        make_kpi_line(
            "Avg time:",
            &format_duration(summary.avg_time_ms),
            theme::TOTAL_TIME_COLOR,
        ),
    ]
}

fn trend_summary_line(app: &App) -> Line<'static> {
    let daily = &app.cache.sparkline;
    let today = daily.last().copied().unwrap_or(0);

    if daily.is_empty() {
        return Line::from(vec![
            summary_label_span("Trend:"),
            Span::styled("collecting data", Style::default().fg(theme::MUTED_COLOR)),
        ]);
    }

    let baseline_values: Vec<u64> = daily
        .iter()
        .rev()
        .skip(1)
        .take(layout::DASHBOARD_TREND_LOOKBACK_DAYS)
        .copied()
        .collect();

    if baseline_values.is_empty() {
        return Line::from(vec![
            summary_label_span("Trend:"),
            Span::styled(
                "building baseline",
                Style::default().fg(theme::HEADER_COLOR),
            ),
        ]);
    }

    let baseline_avg = baseline_values.iter().sum::<u64>() as f64 / baseline_values.len() as f64;

    if baseline_avg == 0.0 {
        let text = if today == 0 {
            "no recent activity".to_string()
        } else {
            format!("new activity · {} today", format_u64_tokens(today))
        };
        return Line::from(vec![
            summary_label_span("Trend:"),
            Span::styled(text, Style::default().fg(theme::HEADER_COLOR)),
        ]);
    }

    let delta_pct = ((today as f64 - baseline_avg) / baseline_avg) * 100.0;
    let (marker, color) = if delta_pct >= 10.0 {
        ("↑", theme::SAVED_COLOR)
    } else if delta_pct <= -10.0 {
        ("↓", theme::ERROR_COLOR)
    } else {
        ("→", theme::HEADER_COLOR)
    };

    Line::from(vec![
        summary_label_span("Trend:"),
        Span::styled(
            format!(
                "{marker} {delta_pct:+.0}% vs 7d avg ({})",
                format_u64_tokens(baseline_avg.round() as u64)
            ),
            Style::default().fg(color),
        ),
    ])
}

fn summary_label_span(label: &str) -> Span<'static> {
    Span::styled(
        format!("{label:<width$}", width = SUMMARY_LABEL_WIDTH),
        Style::default().fg(theme::MUTED_COLOR),
    )
}

fn make_kpi_line(label: &str, value: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        summary_label_span(label),
        Span::styled(value.to_string(), theme::bold(color)),
    ])
}

/// Bar characters for 8 height levels (index 0 = lowest, 7 = tallest).
const BAR_CHARS: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

/// Map a data value to a color based on its percentile of the max.
/// Peak → muted red, High → muted amber, Medium → base_color, Low → DarkGray
fn value_to_color(val: u64, max_val: u64, base_color: Color) -> Color {
    if val == 0 || max_val == 0 {
        return theme::MUTED_COLOR;
    }
    let ratio = val as f64 / max_val as f64;
    if ratio >= 0.9 {
        theme::CHART_PEAK_COLOR
    } else if ratio >= 0.6 {
        theme::CHART_HIGH_COLOR
    } else if ratio >= 0.2 {
        base_color
    } else {
        theme::MUTED_COLOR
    }
}

/// Build multi-line colored bar chart. Bars grow upward from bottom (adjacent to axis).
/// `content_height` = number of text lines for bars.
/// Total resolution = content_height * 8 levels.
fn build_bar_lines(data: &[u64], base_color: Color, content_height: usize) -> Vec<Line<'static>> {
    let max_val = data.iter().copied().max().unwrap_or(0);
    let total_levels = content_height * 8;

    let levels: Vec<usize> = data
        .iter()
        .map(|&v| {
            if v == 0 || max_val == 0 {
                0
            } else {
                let l = ((v as f64 / max_val as f64) * total_levels as f64).round() as usize;
                l.clamp(1, total_levels)
            }
        })
        .collect();

    (0..content_height)
        .map(|row| {
            let base_level = (content_height - 1 - row) * 8;
            let spans: Vec<Span<'static>> = levels
                .iter()
                .zip(data.iter())
                .map(|(&level, &val)| {
                    let color = value_to_color(val, max_val, base_color);
                    if level <= base_level {
                        Span::styled(" ".to_string(), Style::default())
                    } else if level >= base_level + 8 {
                        Span::styled("█".to_string(), Style::default().fg(color))
                    } else {
                        let sub = level - base_level;
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
pub(crate) fn stretch_data(data: &[u64], target_width: usize) -> Vec<u64> {
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

fn stretched_chart_data<'a>(
    cached: &'a [u64],
    cached_width: usize,
    raw: &'a [u64],
    width: usize,
) -> Cow<'a, [u64]> {
    if cached_width == width && !cached.is_empty() {
        Cow::Borrowed(cached)
    } else {
        Cow::Owned(stretch_data(raw, width))
    }
}

fn render_last_24h(frame: &mut Frame, app: &App, area: Rect) {
    let title = chart_title(
        "Last 24 Hours",
        Some(app.cache.saved_last_24h),
        peak_value(&app.cache.sparkline_24h),
        area.width,
    );

    let chart_height = area
        .height
        .saturating_sub(layout::DASHBOARD_CHART_AXIS_HEIGHT * 2)
        .max(1);
    let inner = Layout::vertical([
        Constraint::Length(chart_height),
        Constraint::Length(layout::DASHBOARD_CHART_AXIS_HEIGHT),
        Constraint::Length(layout::DASHBOARD_CHART_AXIS_HEIGHT),
    ])
    .split(area);

    let spark_width = inner[0].width.saturating_sub(2) as usize;
    let stretched = stretched_chart_data(
        app.stretched_sparkline_24h(),
        app.chart_cache_width(),
        &app.cache.sparkline_24h,
        spark_width,
    );
    let content_h = inner[0].height.saturating_sub(1) as usize;
    let bar_lines = build_bar_lines(stretched.as_ref(), theme::CHART_24H_COLOR, content_h);

    let paragraph = Paragraph::new(bar_lines).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(title),
    );
    frame.render_widget(paragraph, inner[0]);

    let axis = build_hour_axis(inner[1].width as usize);
    let paragraph = Paragraph::new(axis);
    frame.render_widget(paragraph, inner[1]);

    let bottom = Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT);
    frame.render_widget(bottom, inner[2]);
}

fn chart_title(label: &str, total_saved: Option<i64>, peak: u64, width: u16) -> String {
    let peak_text = format_u64_tokens(peak);
    match total_saved {
        Some(total_saved) if width >= layout::CHART_TITLE_WIDE_MIN_WIDTH => format!(
            " {label} — Saved {} · Peak {peak_text} ",
            format_tokens(total_saved)
        ),
        Some(total_saved) if width >= layout::CHART_TITLE_MEDIUM_MIN_WIDTH => {
            format!(
                " {label} · {} · Pk {peak_text} ",
                format_tokens(total_saved)
            )
        }
        Some(total_saved) => format!(" {label} {} ", format_tokens(total_saved)),
        None if width >= layout::CHART_TITLE_WIDE_MIN_WIDTH => {
            format!(" {label} — Peak {peak_text} ")
        }
        None if width >= layout::CHART_TITLE_MEDIUM_MIN_WIDTH => {
            format!(" {label} · Pk {peak_text} ")
        }
        None => format!(" {label} "),
    }
}

fn peak_value(data: &[u64]) -> u64 {
    data.iter().copied().max().unwrap_or(0)
}

fn format_u64_tokens(value: u64) -> String {
    format_tokens(value.min(i64::MAX as u64) as i64)
}

/// Build hour axis label line with hour-aligned clock times and a trailing "now".
fn build_hour_axis(width: usize) -> Line<'static> {
    let inner_w = width.saturating_sub(2);
    if inner_w < 10 {
        return Line::from("");
    }

    let now = chrono::Local::now();
    let aligned_now = now
        .with_minute(0)
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .unwrap_or(now);
    let max_labels = (inner_w / 5).max(2);
    let step = [2usize, 4, 6, 8, 12, 24]
        .into_iter()
        .find(|step| 24 / *step < max_labels)
        .unwrap_or(24);
    let count = 24 / step + 1;
    let labels: Vec<String> = (0..count)
        .map(|i| {
            if i + 1 == count {
                "now".to_string()
            } else {
                let hours_ago = 24 - i * step;
                let t = aligned_now - chrono::Duration::hours(hours_ago as i64);
                t.format("%H:%M").to_string()
            }
        })
        .collect();
    wrap_axis_in_border(&labels, inner_w)
}

fn render_sparkline(frame: &mut Frame, app: &App, area: Rect) {
    let title = chart_title(
        "Last 30 Days",
        None,
        peak_value(&app.cache.sparkline),
        area.width,
    );

    let chart_height = area
        .height
        .saturating_sub(layout::DASHBOARD_CHART_AXIS_HEIGHT * 2)
        .max(1);
    let inner = Layout::vertical([
        Constraint::Length(chart_height),
        Constraint::Length(layout::DASHBOARD_CHART_AXIS_HEIGHT),
        Constraint::Length(layout::DASHBOARD_CHART_AXIS_HEIGHT),
    ])
    .split(area);

    let spark_width = inner[0].width.saturating_sub(2) as usize;
    let stretched = stretched_chart_data(
        app.stretched_sparkline_30d(),
        app.chart_cache_width(),
        &app.cache.sparkline,
        spark_width,
    );
    let content_h = inner[0].height.saturating_sub(1) as usize;
    let bar_lines = build_bar_lines(stretched.as_ref(), theme::CHART_30D_COLOR, content_h);

    let paragraph = Paragraph::new(bar_lines).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(title),
    );
    frame.render_widget(paragraph, inner[0]);

    let axis = build_day_axis(inner[1].width as usize);
    let paragraph = Paragraph::new(axis);
    frame.render_widget(paragraph, inner[1]);

    let bottom = Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT);
    frame.render_widget(bottom, inner[2]);
}

/// Build day axis label line with a regular weekly cadence.
fn build_day_axis(width: usize) -> Line<'static> {
    let inner_w = width.saturating_sub(2);
    if inner_w < 10 {
        return Line::from("");
    }

    let today = chrono::Local::now().date_naive();
    let max_labels = (inner_w / 6).max(2);
    let step = [7i64, 14, 28]
        .into_iter()
        .find(|step| 28 / *step < max_labels as i64)
        .unwrap_or(28);
    let count = (28 / step + 1) as usize;
    let labels: Vec<String> = (0..count)
        .map(|i| {
            let days_ago = ((count - 1 - i) as i64) * step;
            if days_ago == 0 {
                "now".to_string()
            } else {
                let d = today - chrono::Duration::days(days_ago);
                if inner_w < 56 {
                    d.format("%m/%d")
                        .to_string()
                        .trim_start_matches('0')
                        .to_string()
                } else {
                    d.format("%m/%d").to_string()
                }
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
    let style = Style::default().fg(theme::MUTED_COLOR);
    let border_style = Style::default().fg(theme::TEXT_COLOR);

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
    let inner_width = area.width.saturating_sub(2) as usize;
    let mut lines = vec![
        recent_header_line(inner_width),
        recent_divider_line(inner_width),
    ];
    lines.extend(
        app.cache
            .recent
            .iter()
            .map(|record| build_recent_line(record, inner_width)),
    );

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Recent Commands "),
    );

    frame.render_widget(paragraph, area);
}

fn build_recent_line(record: &db::CommandRecord, inner_width: usize) -> Line<'static> {
    let command = sanitize(&record.rtk_cmd);
    let exec = format_duration(record.exec_time_ms);
    let saved = format_tokens(record.saved_tokens);
    let ts_short: String = record.timestamp.chars().skip(11).take(5).collect();

    if inner_width >= layout::RECENT_WIDE_MIN_WIDTH as usize {
        let fixed_width = 2 + 8 + 2 + 8 + 2 + 5;
        let cmd_width = inner_width.saturating_sub(fixed_width).max(12);
        let command = truncate_cmd(&command, cmd_width);

        Line::from(vec![
            recent_cell(&command, cmd_width, theme::TEXT_COLOR, false),
            Span::raw("  "),
            recent_bold_cell(&saved, 8, theme::SAVED_COLOR, true),
            Span::raw("  "),
            recent_cell(&exec, 8, theme::TOTAL_TIME_COLOR, true),
            Span::raw("  "),
            recent_cell(&ts_short, 5, theme::MUTED_COLOR, true),
        ])
    } else if inner_width >= layout::RECENT_MEDIUM_MIN_WIDTH as usize {
        let fixed_width = 2 + 8 + 2 + 8;
        let cmd_width = inner_width.saturating_sub(fixed_width).max(12);
        let command = truncate_cmd(&command, cmd_width);

        Line::from(vec![
            recent_cell(&command, cmd_width, theme::TEXT_COLOR, false),
            Span::raw("  "),
            recent_bold_cell(&saved, 8, theme::SAVED_COLOR, true),
            Span::raw("  "),
            recent_cell(&exec, 8, theme::TOTAL_TIME_COLOR, true),
        ])
    } else {
        let fixed_width = 2 + 8;
        let cmd_width = inner_width.saturating_sub(fixed_width).max(10);
        let command = truncate_cmd(&command, cmd_width);

        Line::from(vec![
            recent_cell(&command, cmd_width, theme::TEXT_COLOR, false),
            Span::raw("  "),
            recent_bold_cell(&saved, 8, theme::SAVED_COLOR, true),
        ])
    }
}

fn recent_header_line(inner_width: usize) -> Line<'static> {
    if inner_width >= layout::RECENT_WIDE_MIN_WIDTH as usize {
        let fixed_width = 2 + 8 + 2 + 8 + 2 + 5;
        let cmd_width = inner_width.saturating_sub(fixed_width).max(12);
        Line::from(vec![
            recent_cell("Command", cmd_width, theme::MUTED_COLOR, false),
            Span::raw("  "),
            recent_cell("Saved", 8, theme::MUTED_COLOR, true),
            Span::raw("  "),
            recent_cell("Exec", 8, theme::MUTED_COLOR, true),
            Span::raw("  "),
            recent_cell("Time", 5, theme::MUTED_COLOR, true),
        ])
    } else if inner_width >= layout::RECENT_MEDIUM_MIN_WIDTH as usize {
        let fixed_width = 2 + 8 + 2 + 8;
        let cmd_width = inner_width.saturating_sub(fixed_width).max(12);
        Line::from(vec![
            recent_cell("Command", cmd_width, theme::MUTED_COLOR, false),
            Span::raw("  "),
            recent_cell("Saved", 8, theme::MUTED_COLOR, true),
            Span::raw("  "),
            recent_cell("Exec", 8, theme::MUTED_COLOR, true),
        ])
    } else {
        let fixed_width = 2 + 8;
        let cmd_width = inner_width.saturating_sub(fixed_width).max(10);
        Line::from(vec![
            recent_cell("Command", cmd_width, theme::MUTED_COLOR, false),
            Span::raw("  "),
            recent_cell("Saved", 8, theme::MUTED_COLOR, true),
        ])
    }
}

fn recent_divider_line(inner_width: usize) -> Line<'static> {
    Line::from(Span::styled(
        "─".repeat(inner_width.max(8)),
        Style::default().fg(theme::MUTED_COLOR),
    ))
}

fn recent_cell(text: &str, width: usize, color: Color, right_align: bool) -> Span<'static> {
    let padded = if right_align {
        format!("{text:>width$}")
    } else {
        format!("{text:<width$}")
    };
    Span::styled(padded, Style::default().fg(color))
}

fn recent_bold_cell(text: &str, width: usize, color: Color, right_align: bool) -> Span<'static> {
    let padded = if right_align {
        format!("{text:>width$}")
    } else {
        format!("{text:<width$}")
    };
    Span::styled(padded, theme::bold(color))
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

    #[test]
    fn test_chart_title_compact() {
        assert_eq!(
            chart_title("Last 30 Days", None, 1200, 40),
            " Last 30 Days · Pk 1.2K "
        );
    }

    #[test]
    fn test_peak_value() {
        assert_eq!(peak_value(&[1, 9, 3]), 9);
        assert_eq!(peak_value(&[]), 0);
    }
}

#[cfg(test)]
mod render_tests {
    use crate::app::App;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn make_test_app() -> App {
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
        )
        .unwrap();
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
        let content: String = (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

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
