use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::api::story::Story;

/// Renders a single story item with divider at the bottom
pub struct StoryItemWidget<'a> {
    story: &'a Story,
    is_active: bool,
    is_selected: bool,
    next_is_selected: bool,
    _width: u16,
    is_completed: bool,
}

impl<'a> StoryItemWidget<'a> {
    pub fn new(
        story: &'a Story,
        is_active: bool,
        is_selected: bool,
        next_is_selected: bool,
        width: u16,
        is_completed: bool,
    ) -> Self {
        Self {
            story,
            is_active,
            is_selected,
            next_is_selected,
            _width: width,
            is_completed,
        }
    }

    /// Calculate the total height including divider
    pub fn height(&self) -> u16 {
        // Story content + divider
        2
    }
}

impl Widget for StoryItemWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }

        // Render story content on first line
        let content = self.render_story_line();
        buf.set_line(area.x, area.y, &content, area.width);

        // Render divider on second line
        // Use yellow if this story is selected OR if the next story is selected
        let divider_style = if self.is_selected || self.next_is_selected {
            Style::default().fg(Color::Yellow)
        } else if self.is_completed {
            Style::default().gray()
        } else {
            Style::default()
        };
        let divider = Line::from("─".repeat(area.width as usize)).style(divider_style);
        buf.set_line(area.x, area.y + 1, &divider, area.width);
    }
}

impl StoryItemWidget<'_> {
    fn render_story_line(&self) -> Line<'static> {
        let mut spans = Vec::new();

        // Base style: gray if completed
        let base_style = if self.is_completed {
            Style::default().gray()
        } else {
            Style::default()
        };

        // Active indicator
        if self.is_active {
            let color = if self.is_completed {
                Color::DarkGray
            } else {
                Color::Green
            };
            spans.push(Span::styled("● ", Style::default().fg(color)));
        } else {
            spans.push(Span::raw("  "));
        }

        // Story ID
        let id_color = if self.is_completed {
            Color::DarkGray
        } else {
            Color::Blue
        };
        spans.push(Span::styled(
            format!("sc-{} ", self.story.id),
            Style::default().fg(id_color),
        ));

        // Story name (apply bold if selected)
        let name_style = if self.is_selected {
            base_style.reversed()
        } else {
            base_style
        };
        spans.push(Span::styled(self.story.name.clone(), name_style));

        Line::from(spans)
    }
}
