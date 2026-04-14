use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use super::{format_number, sanitize, shorten_path};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let data = &app.cache.projects;
    let header = Row::new(vec!["Project", "Commands", "Saved", "Savings %"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = data
        .iter()
        .skip(app.scroll_offset)
        .map(|p| {
            Row::new(vec![
                Cell::from(sanitize(&shorten_path(&p.project_path, 40))),
                Cell::from(format_number(p.commands)),
                Cell::from(format_number(p.total_saved)).style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.1}%", p.savings_pct))
                    .style(Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();

    let indicator = if data.is_empty() {
        String::new()
    } else {
        format!(" [{}/{}]", app.scroll_offset + 1, data.len())
    };
    let title = format!(" Projects [j/k to scroll]{indicator} ");

    let table = Table::new(
        rows,
        [
            Constraint::Min(40),
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}
