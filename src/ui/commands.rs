use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use super::{format_number, format_tokens, sanitize};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let data = &app.cache.top_commands;
    let header = Row::new(vec!["Command", "Count", "Total Saved", "Avg Savings %"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = data
        .iter()
        .skip(app.scroll_offset)
        .map(|c| {
            Row::new(vec![
                Cell::from(sanitize(c.command.as_str())),
                Cell::from(format_number(c.count)),
                Cell::from(format_tokens(c.total_saved)).style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.1}%", c.avg_savings_pct))
                    .style(Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();

    let indicator = if data.is_empty() {
        String::new()
    } else {
        format!(" [{}/{}]", app.scroll_offset + 1, data.len())
    };
    let title = format!(" Top Commands by Tokens Saved [j/k to scroll]{indicator} ");

    let table = Table::new(
        rows,
        [
            Constraint::Min(30),
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}
