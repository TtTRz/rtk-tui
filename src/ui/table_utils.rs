use ratatui::{
    layout::{Constraint, Rect},
    widgets::Row,
};

use super::{layout, theme};

pub fn make_header<const N: usize>(titles: [&'static str; N]) -> Row<'static> {
    Row::new(titles).style(theme::table_header_style())
}

pub fn scroll_indicator(offset: usize, total: usize) -> String {
    if total == 0 {
        String::new()
    } else {
        format!(" [{}/{}]", offset + 1, total)
    }
}

pub fn search_hint(query: &str) -> Option<String> {
    if query.is_empty() {
        None
    } else {
        Some(format!(" \"{query}\""))
    }
}

pub fn build_title(
    base: &str,
    controls: &str,
    offset: usize,
    total: usize,
    suffix: Option<&str>,
) -> String {
    let indicator = scroll_indicator(offset, total);
    let suffix = suffix.unwrap_or("");

    if controls.is_empty() {
        format!(" {base}{indicator}{suffix} ")
    } else {
        format!(" {base} [{controls}]{indicator}{suffix} ")
    }
}

pub fn visible_row_range(selected: usize, total: usize, area: Rect) -> (usize, usize) {
    let visible_rows = area.height.saturating_sub(3) as usize;
    if total == 0 || visible_rows == 0 {
        return (0, 0);
    }

    let start = selected.saturating_sub(visible_rows.saturating_sub(1));
    let end = (start + visible_rows).min(total);
    (start, end)
}

pub fn command_column_widths(area: Rect) -> Vec<Constraint> {
    let compact = area.width < 90;
    vec![
        Constraint::Min(if compact {
            18
        } else {
            layout::COMMAND_NAME_MIN_WIDTH
        }),
        Constraint::Length(if compact {
            8
        } else {
            layout::COUNT_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            11
        } else {
            layout::WIDE_VALUE_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            11
        } else {
            layout::WIDE_VALUE_COLUMN_WIDTH
        }),
    ]
}

pub fn project_column_widths(area: Rect) -> Vec<Constraint> {
    let compact = area.width < 96;
    vec![
        Constraint::Min(if compact {
            20
        } else {
            layout::PROJECT_NAME_MIN_WIDTH
        }),
        Constraint::Length(if compact {
            8
        } else {
            layout::COUNT_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            11
        } else {
            layout::WIDE_VALUE_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            11
        } else {
            layout::WIDE_VALUE_COLUMN_WIDTH
        }),
    ]
}

pub fn daily_column_widths(area: Rect) -> Vec<Constraint> {
    let compact = area.width < 92;
    vec![
        Constraint::Length(if compact {
            10
        } else {
            layout::DAILY_DATE_WIDTH
        }),
        Constraint::Length(if compact {
            8
        } else {
            layout::COUNT_COLUMN_WIDTH
        }),
        Constraint::Min(if compact {
            8
        } else {
            layout::TOKENS_COLUMN_WIDTH
        }),
        Constraint::Min(if compact {
            8
        } else {
            layout::TOKENS_COLUMN_WIDTH
        }),
        Constraint::Min(if compact {
            8
        } else {
            layout::TOKENS_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            9
        } else {
            layout::PERCENT_COLUMN_WIDTH
        }),
    ]
}

pub fn weekly_column_widths(area: Rect) -> Vec<Constraint> {
    let compact = area.width < 88;
    vec![
        Constraint::Min(if compact {
            18
        } else {
            layout::WEEK_RANGE_WIDTH
        }),
        Constraint::Length(if compact {
            8
        } else {
            layout::COUNT_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            10
        } else {
            layout::TOKENS_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            9
        } else {
            layout::PERCENT_COLUMN_WIDTH
        }),
    ]
}

pub fn monthly_column_widths(area: Rect) -> Vec<Constraint> {
    let compact = area.width < 72;
    vec![
        Constraint::Length(if compact {
            8
        } else {
            layout::COUNT_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            8
        } else {
            layout::COUNT_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            10
        } else {
            layout::TOKENS_COLUMN_WIDTH
        }),
        Constraint::Length(if compact {
            9
        } else {
            layout::PERCENT_COLUMN_WIDTH
        }),
    ]
}
