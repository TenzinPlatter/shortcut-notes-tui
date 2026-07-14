use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect, Size},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget, Wrap},
};
use tui_scrollview::{ScrollView, ScrollViewState};

use crate::api::story::Story;

pub struct DescriptionModal<'a> {
    story: &'a Story,
}

impl<'a> DescriptionModal<'a> {
    pub fn new(story: &'a Story) -> Self {
        Self { story }
    }
}

impl StatefulWidget for DescriptionModal<'_> {
    type State = ScrollViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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

        // Description content with word wrap via ScrollView
        let content_area = chunks[2];
        let trimmed = self.story.description.trim();
        let description = if trimmed.is_empty() {
            "No description".to_string()
        } else {
            trimmed.to_string()
        };

        let paragraph = Paragraph::new(Text::from(description)).wrap(Wrap { trim: false });

        let content_width = content_area.width;
        let total_lines = paragraph.line_count(content_width) as u16;

        let mut scroll_view = ScrollView::new(Size::new(content_width, total_lines));
        scroll_view.render_widget(paragraph, Rect::new(0, 0, content_width, total_lines));
        scroll_view.render(content_area, buf, state);

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
