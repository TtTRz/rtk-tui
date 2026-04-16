use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Cell, Row, Table},
};

use super::{format_number, format_tokens, sanitize, table_utils, theme};
use crate::app::App;

fn truncate_command(command: &str, max_len: usize) -> String {
    let chars: Vec<char> = command.chars().collect();
    if chars.len() <= max_len {
        command.to_string()
    } else {
        let mut truncated: String = chars[..max_len.saturating_sub(1)].iter().collect();
        truncated.push('…');
        truncated
    }
}

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let query = app.search_query.to_lowercase();
    let data: Vec<_> = app
        .cache
        .top_commands
        .iter()
        .filter(|c| query.is_empty() || c.command.to_lowercase().contains(&query))
        .collect();

    let selected = app.scroll_offset();
    let (start, end) = table_utils::visible_row_range(selected, data.len(), area);
    let text_width = area.width.saturating_sub(36).max(12) as usize;
    let header = table_utils::make_header(["Command", "Count", "Total Saved", "Avg Savings %"]);

    let rows: Vec<Row> = data[start..end]
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            let absolute_idx = start + idx;
            let mut row = Row::new(vec![
                Cell::from(truncate_command(&sanitize(c.command.as_str()), text_width)),
                Cell::from(format_number(c.count)),
                Cell::from(format_tokens(c.total_saved))
                    .style(Style::default().fg(theme::SAVED_COLOR)),
                Cell::from(format!("{:.1}%", c.avg_savings_pct))
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
        "Top Commands by Tokens Saved",
        "j/k select · / search",
        selected,
        data.len(),
        search_hint.as_deref(),
    );

    let table = Table::new(rows, table_utils::command_column_widths(area))
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}
