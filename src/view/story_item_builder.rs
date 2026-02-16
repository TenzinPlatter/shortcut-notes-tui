use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::api::story::Story;

/// Renders a single story item with divider at the bottom
pub struct StoryItemWidget<'a> {
    story: &'a Story,
    is_active: bool,
    is_selected: bool,
    highlight_style: Style,
    _width: u16,
}

impl<'a> StoryItemWidget<'a> {
    pub fn new(story: &'a Story, is_active: bool, is_selected: bool, highlight_style: Style, width: u16) -> Self {
        Self {
            story,
            is_active,
            is_selected,
            highlight_style,
            _width: width,
        }
    }

    /// Calculate the total height including divider
    pub fn height(&self) -> u16 {
        // Story content is 1 line + 1 line for divider
        2
    }
}

impl Widget for StoryItemWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }

        // Apply background highlight to the entire first line if selected
        if self.is_selected {
            for x in area.x..area.x + area.width {
                buf[(x, area.y)].set_style(self.highlight_style);
            }
        }

        // Render story content on first line
        let content = self.render_story_line();
        buf.set_line(area.x, area.y, &content, area.width);

        // Render divider on second line
        let divider = Line::from("─".repeat(area.width as usize));
        buf.set_line(area.x, area.y + 1, &divider, area.width);
    }
}

impl StoryItemWidget<'_> {
    fn render_story_line(&self) -> Line<'static> {
        let mut spans = Vec::new();

        // Active indicator
        if self.is_active {
            spans.push(Span::styled("● ", Style::default().fg(Color::Green)));
        } else {
            spans.push(Span::raw("  "));
        }

        // Story ID
        spans.push(Span::styled(
            format!("sc-{} ", self.story.id),
            Style::default().fg(Color::Cyan),
        ));

        // Story name
        let name_style = if self.is_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        spans.push(Span::styled(self.story.name.clone(), name_style));

        Line::from(spans)
    }
}
