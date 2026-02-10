use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};

use crate::api::story::Story;

pub struct DescriptionModal<'a> {
    story: &'a Story,
    scroll_offset: u16,
}

impl<'a> DescriptionModal<'a> {
    pub fn new(story: &'a Story, scroll_offset: u16) -> Self {
        Self {
            story,
            scroll_offset,
        }
    }

    /// Calculate the number of wrapped lines for the description given a width.
    /// Used for scroll bounds calculation.
    pub fn calculate_total_lines(description: &str, wrap_width: u16) -> u16 {
        if wrap_width == 0 {
            return 0;
        }

        let trimmed = description.trim();
        if trimmed.is_empty() {
            return 1; // "No description" placeholder
        }

        let mut total_lines = 0u16;
        for line in trimmed.lines() {
            if line.is_empty() {
                total_lines += 1;
            } else {
                // Approximate wrapped line count
                let chars = line.chars().count() as u16;
                total_lines += (chars / wrap_width) + 1;
            }
        }

        total_lines.max(1)
    }
}

impl Widget for DescriptionModal<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Layout: title bar, divider, content, divider, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Length(1), // Divider
                Constraint::Min(1),    // Content
                Constraint::Length(1), // Divider
                Constraint::Length(1), // Footer
            ])
            .margin(1)
            .split(area);

        // Outer block with border
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_bottom(" j/k scroll • g/G top/bottom • q close ");

        block.render(area, buf);

        // Title (story name)
        let title = Line::from(self.story.name.as_str())
            .style(Style::default().add_modifier(Modifier::BOLD));
        buf.set_line(chunks[0].x, chunks[0].y, &title, chunks[0].width);

        // Top divider
        let divider = "─".repeat(chunks[1].width as usize);
        buf.set_string(chunks[1].x, chunks[1].y, &divider, Style::default());

        // Description content with word wrap and scroll
        let trimmed = self.story.description.trim();
        let description = if trimmed.is_empty() {
            "No description".to_string()
        } else {
            trimmed.to_string()
        };

        let paragraph = Paragraph::new(Text::from(description))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0));

        paragraph.render(chunks[2], buf);

        // Bottom divider
        buf.set_string(chunks[3].x, chunks[3].y, &divider, Style::default());
    }
}

/// Calculate a centered rectangle with percentage-based sizing
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = (area.width * percent_x) / 100;
    let height = (area.height * percent_y) / 100;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    Rect::new(x, y, width, height)
}

/// Get the content area height (for scroll calculations)
pub fn content_height(modal_area: Rect) -> u16 {
    // Modal area minus: border (2) + title (1) + dividers (2) + footer (1) + margins (2)
    modal_area.height.saturating_sub(8)
}

/// Get the content area width (for line wrap calculations)
pub fn content_width(modal_area: Rect) -> u16 {
    // Modal area minus: border (2) + margins (2)
    modal_area.width.saturating_sub(4)
}
