//! Buddy-specific rendering logic — sprite positioning, bubble layout.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::Mood;
use crate::app::App;

/// Render the buddy panel (sprite + speech bubble) into the given area.
pub fn render_buddy(frame: &mut Frame, app: &App, area: Rect) {
    let buddy = &app.buddy;
    let inner_w = area.width.saturating_sub(2) as usize;
    let sprite_lines = buddy.current_frame();

    let mut lines: Vec<Line> = Vec::new();
    let pad = " ".repeat(buddy.x_pos);

    // Speech bubble (follows x_pos)
    if let Some(text) = &buddy.bubble_text {
        let max_text_w = inner_w.saturating_sub(buddy.x_pos + 4);
        let text = if text.len() > max_text_w {
            format!("{}…", &text[..max_text_w.saturating_sub(1).max(1)])
        } else {
            text.clone()
        };
        let bubble_w = text.len() + 2;
        lines.push(Line::from(Span::styled(
            format!("{pad} .{}.", "-".repeat(bubble_w)),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            format!("{pad} | {text} |"),
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(Span::styled(
            format!("{pad} `{}'", "-".repeat(bubble_w)),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            format!("{pad}       \\"),
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for _ in 0..4 {
            lines.push(Line::from(""));
        }
    }

    // Bounce: remove one empty/bubble line to shift sprite up
    if buddy.bounce_phase && lines.len() > 1 {
        lines.remove(0);
    }

    // Sprite with x_pos offset
    let sprite_color = mood_color(buddy.mood);
    for line in &sprite_lines {
        lines.push(Line::from(Span::styled(
            format!("{pad}{line}"),
            Style::default().fg(sprite_color),
        )));
    }

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Buddy "));
    frame.render_widget(paragraph, area);
}

/// Map mood to display color.
fn mood_color(mood: Mood) -> Color {
    match mood {
        Mood::Ecstatic => Color::Yellow,
        Mood::Happy => Color::Green,
        Mood::Content => Color::Cyan,
        Mood::Sleepy => Color::Blue,
        Mood::Worried => Color::Red,
    }
}
