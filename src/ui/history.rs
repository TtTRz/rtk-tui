use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Cell, Row, Table},
};

use super::{format_number, format_tokens, table_utils, theme};
use crate::app::{App, HistoryView};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    match app.history_view {
        HistoryView::Daily => render_daily(frame, app, area),
        HistoryView::Weekly => render_weekly(frame, app, area),
        HistoryView::Monthly => render_monthly(frame, app, area),
    }
}

fn render_daily(frame: &mut Frame, app: &App, area: Rect) {
    let data = &app.cache.daily;
    let selected = app.scroll_offset();
    let (start, end) = table_utils::visible_row_range(selected, data.len(), area);
    let header =
        table_utils::make_header(["Date", "Commands", "Input", "Output", "Saved", "Savings %"]);

    let rows: Vec<Row> = data[start..end]
        .iter()
        .enumerate()
        .map(|(idx, d)| {
            let absolute_idx = start + idx;
            let mut row = Row::new(vec![
                Cell::from(d.date.as_str()),
                Cell::from(format_number(d.commands)),
                Cell::from(format_tokens(d.input_tokens)),
                Cell::from(format_tokens(d.output_tokens)),
                Cell::from(format_tokens(d.saved_tokens))
                    .style(Style::default().fg(theme::SAVED_COLOR)),
                Cell::from(format!("{:.1}%", d.savings_pct))
                    .style(Style::default().fg(theme::PERCENTAGE_COLOR)),
            ]);
            if absolute_idx == selected {
                row = row.style(theme::selected_row_style());
            }
            row
        })
        .collect();

    let title = table_utils::build_title(
        "Daily Stats",
        "d/w/m switch · j/k select",
        selected,
        data.len(),
        None,
    );
    let table = Table::new(rows, table_utils::daily_column_widths(area))
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_weekly(frame: &mut Frame, app: &App, area: Rect) {
    let data = &app.cache.weekly;
    let selected = app.scroll_offset();
    let (start, end) = table_utils::visible_row_range(selected, data.len(), area);
    let header = table_utils::make_header(["Week", "Commands", "Saved", "Savings %"]);

    let rows: Vec<Row> = data[start..end]
        .iter()
        .enumerate()
        .map(|(idx, w)| {
            let absolute_idx = start + idx;
            let label = format!("{} → {}", w.week_start, w.week_end);
            let mut row = Row::new(vec![
                Cell::from(label),
                Cell::from(format_number(w.commands)),
                Cell::from(format_tokens(w.saved_tokens))
                    .style(Style::default().fg(theme::SAVED_COLOR)),
                Cell::from(format!("{:.1}%", w.savings_pct))
                    .style(Style::default().fg(theme::PERCENTAGE_COLOR)),
            ]);
            if absolute_idx == selected {
                row = row.style(theme::selected_row_style());
            }
            row
        })
        .collect();

    let title = table_utils::build_title(
        "Weekly Stats",
        "d/w/m switch · j/k select",
        selected,
        data.len(),
        None,
    );
    let table = Table::new(rows, table_utils::weekly_column_widths(area))
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_monthly(frame: &mut Frame, app: &App, area: Rect) {
    let data = &app.cache.monthly;
    let selected = app.scroll_offset();
    let (start, end) = table_utils::visible_row_range(selected, data.len(), area);
    let header = table_utils::make_header(["Month", "Commands", "Saved", "Savings %"]);

    let rows: Vec<Row> = data[start..end]
        .iter()
        .enumerate()
        .map(|(idx, m)| {
            let absolute_idx = start + idx;
            let mut row = Row::new(vec![
                Cell::from(m.month.as_str()),
                Cell::from(format_number(m.commands)),
                Cell::from(format_tokens(m.saved_tokens))
                    .style(Style::default().fg(theme::SAVED_COLOR)),
                Cell::from(format!("{:.1}%", m.savings_pct))
                    .style(Style::default().fg(theme::PERCENTAGE_COLOR)),
            ]);
            if absolute_idx == selected {
                row = row.style(theme::selected_row_style());
            }
            row
        })
        .collect();

    let title = table_utils::build_title(
        "Monthly Stats",
        "d/w/m switch · j/k select",
        selected,
        data.len(),
        None,
    );
    let table = Table::new(rows, table_utils::monthly_column_widths(area))
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}
