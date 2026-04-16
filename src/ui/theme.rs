use ratatui::style::{Color, Modifier, Style};

pub const TEXT_COLOR: Color = Color::White;
pub const MUTED_COLOR: Color = Color::DarkGray;
pub const HEADER_COLOR: Color = Color::Yellow;
pub const SAVED_COLOR: Color = Color::Green;
pub const PERCENTAGE_COLOR: Color = Color::Cyan;
pub const INFO_COLOR: Color = Color::Blue;
pub const ERROR_COLOR: Color = Color::Red;
pub const TOTAL_TIME_COLOR: Color = Color::Magenta;
pub const CHART_24H_COLOR: Color = Color::Cyan;
pub const CHART_30D_COLOR: Color = Color::Green;
pub const CHART_PEAK_COLOR: Color = Color::Rgb(200, 100, 100);
pub const CHART_HIGH_COLOR: Color = Color::Rgb(200, 170, 80);
pub const BUDDY_ECSTATIC_COLOR: Color = Color::Yellow;
pub const BUDDY_HAPPY_COLOR: Color = Color::Green;
pub const BUDDY_CONTENT_COLOR: Color = Color::Cyan;
pub const BUDDY_SLEEPY_COLOR: Color = Color::Blue;
pub const BUDDY_WORRIED_COLOR: Color = Color::Red;

pub fn bold(color: Color) -> Style {
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

pub fn table_header_style() -> Style {
    bold(HEADER_COLOR)
}

pub fn selected_row_style() -> Style {
    Style::default().fg(TEXT_COLOR).bg(MUTED_COLOR)
}

pub fn tab_highlight_style() -> Style {
    bold(SAVED_COLOR)
}

pub fn status_style(background: Color) -> Style {
    Style::default().fg(TEXT_COLOR).bg(background)
}

pub fn passive_status_style() -> Style {
    Style::default().fg(MUTED_COLOR)
}
