use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Cell, Row, Table},
};

use super::{format_number, format_tokens, sanitize, shorten_path, table_utils, theme};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let query = app.search_query.to_lowercase();
    let data: Vec<_> = app
        .cache
        .projects
        .iter()
        .filter(|p| query.is_empty() || p.project_path.to_lowercase().contains(&query))
        .collect();

    let selected = app.scroll_offset();
    let (start, end) = table_utils::visible_row_range(selected, data.len(), area);
    let path_width = area.width.saturating_sub(34).max(14) as usize;
    let header = table_utils::make_header(["Project", "Commands", "Saved", "Savings %"]);

    let rows: Vec<Row> = data[start..end]
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let absolute_idx = start + idx;
            let mut row = Row::new(vec![
                Cell::from(sanitize(&shorten_path(&p.project_path, path_width))),
                Cell::from(format_number(p.commands)),
                Cell::from(format_tokens(p.total_saved))
                    .style(Style::default().fg(theme::SAVED_COLOR)),
                Cell::from(format!("{:.1}%", p.savings_pct))
                    .style(Style::default().fg(theme::PERCENTAGE_COLOR)),
            ]);
            if absolute_idx == selected {
                row = row.style(theme::selected_row_style());
            }
            row
        })
        .collect();

    let search_hint = table_utils::search_hint(&query);
    let title = table_utils::build_title(
        "Projects",
        "j/k select · / search",
        selected,
        data.len(),
        search_hint.as_deref(),
    );

    let table = Table::new(rows, table_utils::project_column_widths(area))
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}
