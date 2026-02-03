use chrono::{Duration, NaiveDateTime, Utc};
use ratatui::{
    buffer::Buffer,
    layout::{HorizontalAlignment, Rect},
    style::Style,
    widgets::{Block, BorderType, Paragraph, Widget, WidgetRef, Wrap},
};

use crate::text_utils::{count_wrapped_lines, truncate_to_lines};

const ERROR_NOTIFICATION_WINDOW_MAX_WIDTH: u16 = 50;
const ERROR_NOTIFICATION_MAX_TEXT_HEIGHT: u16 = 3;
pub const ERROR_NOTIFICATION_MAX_HEIGHT: u16 = ERROR_NOTIFICATION_MAX_TEXT_HEIGHT + 2; // +2 for borders

#[derive(Clone, Debug)]
pub struct ErrorInfo {
    short: String,
    long: String,
    created_at: NaiveDateTime,
}

impl ErrorInfo {
    pub fn get_required_width(&self) -> u16 {
        let text_len = u16::max(self.short.len() as u16, self.long.len() as u16);

        // +2 for border chars, +2 for padding
        u16::min(text_len, ERROR_NOTIFICATION_WINDOW_MAX_WIDTH)
    }

    pub fn get_required_height(&self, available_width: u16) -> u16 {
        let inner_width = available_width.saturating_sub(2) as usize;
        if inner_width == 0 {
            return ERROR_NOTIFICATION_MAX_HEIGHT;
        }

        let line_count = count_wrapped_lines(&self.long, inner_width);
        let text_height = (line_count as u16).min(ERROR_NOTIFICATION_MAX_TEXT_HEIGHT);

        text_height + 2
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().naive_utc() > self.created_at + Duration::seconds(3)
    }

    pub fn new<A, B>(short: A, long: B) -> ErrorInfo
    where
        A: ToString,
        B: ToString,
    {
        ErrorInfo {
            short: short.to_string(),
            long: long.to_string(),
            created_at: Utc::now().naive_utc(),
        }
    }
}

impl WidgetRef for ErrorInfo {
    #[doc = " Draws the current state of the widget in the given buffer. That is the only method required"]
    #[doc = " to implement a custom widget."]
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let truncated_title = truncate(&self.short, ERROR_NOTIFICATION_WINDOW_MAX_WIDTH as usize);
        let inner_width = area.width.saturating_sub(2) as usize;
        let display_text = truncate_to_lines(
            &self.long,
            inner_width,
            ERROR_NOTIFICATION_MAX_TEXT_HEIGHT as usize,
        );

        let block = Block::bordered()
            .title_top(truncated_title)
            .title_alignment(HorizontalAlignment::Center)
            .border_style(Style::default().red())
            .border_type(BorderType::Rounded);

        Paragraph::new(display_text)
            .wrap(Wrap { trim: true })
            .block(block)
            .render(area, buf);
    }
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        ".".repeat(max_width)
    } else {
        format!("{}...", &s[..max_width - 3])
    }
}
