use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
};

use crate::api::story::Story;

use super::list::ListRow;

/// Wrapper around Story that implements ListRow.
/// This allows Story to be rendered in a CustomList while maintaining
/// separation between the data model (Story) and view state (active).
pub struct StoryRow<'a> {
    pub story: &'a Story,
    pub active: bool,
}

impl<'a> StoryRow<'a> {
    pub fn new(story: &'a Story, active: bool) -> Self {
        Self { story, active }
    }
}

impl ListRow for StoryRow<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer, _is_cursor: bool) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let mut y = area.y;
        let max_y = area.y + area.height;

        // Apply active style (reversed) if this is the active story
        let base_style = if self.active {
            Style::default().reversed()
        } else {
            Style::default()
        };

        // Render story name
        if y < max_y {
            let line = Line::from(self.story.name.clone()).style(base_style);
            buf.set_line(area.x, y, &line, area.width);
            y += 1;
        }

        // Render hint to view description
        if y < max_y {
            let line = Line::from("Press <Space> to view description")
                .style(base_style.italic());
            buf.set_line(area.x, y, &line, area.width);
            y += 1;
        }

        // Render empty line at the end
        if y < max_y {
            let line = Line::from("").style(base_style);
            buf.set_line(area.x, y, &line, area.width);
        }
    }

    fn height(&self) -> u16 {
        // Story name (1) + hint (1) + empty line (1)
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_story(name: &str, description: &str) -> Story {
        Story {
            id: 1,
            name: name.to_string(),
            description: description.to_string(),
            completed: false,
            branches: vec![],
            comments: vec![],
            epic_id: None,
            iteration_id: None,
            app_url: "https://app.shortcut.com/test/story/1".to_string(),
        }
    }

    #[test]
    fn test_height() {
        let story = create_test_story("Test Story", "Single line description");
        let row = StoryRow::new(&story, false);
        assert_eq!(row.height(), 3);
    }

    #[test]
    fn test_height_active() {
        let story = create_test_story("Test Story", "Description");
        let row = StoryRow::new(&story, true);
        assert_eq!(row.height(), 3);
    }
}
