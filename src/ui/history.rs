use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use super::{format_number, format_tokens};
use crate::app::{App, HistoryView};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    match app.history_view {
        HistoryView::Daily => render_daily(frame, app, area),
        HistoryView::Weekly => render_weekly(frame, app, area),
        HistoryView::Monthly => render_monthly(frame, app, area),
    }
}

fn scroll_indicator(offset: usize, total: usize) -> String {
    if total == 0 {
        String::new()
    } else {
        format!(" [{}/{}]", offset + 1, total)
    }
}

fn render_daily(frame: &mut Frame, app: &App, area: Rect) {
    let data = &app.cache.daily;
    let header = Row::new(vec![
        "Date",
        "Commands",
        "Input",
        "Output",
        "Saved",
        "Savings %",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = data
        .iter()
        .skip(app.scroll_offset)
        .map(|d| {
            Row::new(vec![
                Cell::from(d.date.as_str()),
                Cell::from(format_number(d.commands)),
                Cell::from(format_tokens(d.input_tokens)),
                Cell::from(format_tokens(d.output_tokens)),
                Cell::from(format_tokens(d.saved_tokens)).style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.1}%", d.savings_pct))
                    .style(Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();

    let title = format!(
        " Daily Stats [d/w/m to switch · j/k to scroll]{} ",
        scroll_indicator(app.scroll_offset, data.len())
    );
    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_weekly(frame: &mut Frame, app: &App, area: Rect) {
    let data = &app.cache.weekly;
    let header = Row::new(vec!["Week", "Commands", "Saved", "Savings %"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = data
        .iter()
        .skip(app.scroll_offset)
        .map(|w| {
            let label = format!("{} → {}", w.week_start, w.week_end);
            Row::new(vec![
                Cell::from(label),
                Cell::from(format_number(w.commands)),
                Cell::from(format_tokens(w.saved_tokens)).style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.1}%", w.savings_pct))
                    .style(Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();

    let title = format!(
        " Weekly Stats [d/w/m to switch · j/k to scroll]{} ",
        scroll_indicator(app.scroll_offset, data.len())
    );
    let table = Table::new(
        rows,
        [
            Constraint::Length(25),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_monthly(frame: &mut Frame, app: &App, area: Rect) {
    let data = &app.cache.monthly;
    let header = Row::new(vec!["Month", "Commands", "Saved", "Savings %"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = data
        .iter()
        .skip(app.scroll_offset)
        .map(|m| {
            Row::new(vec![
                Cell::from(m.month.as_str()),
                Cell::from(format_number(m.commands)),
                Cell::from(format_tokens(m.saved_tokens)).style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.1}%", m.savings_pct))
                    .style(Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();

    let title = format!(
        " Monthly Stats [d/w/m to switch · j/k to scroll]{} ",
        scroll_indicator(app.scroll_offset, data.len())
    );
    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}
