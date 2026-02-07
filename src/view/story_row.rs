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
/// separation between the data model (Story) and view state (expanded/active).
pub struct StoryRow<'a> {
    pub story: &'a Story,
    pub expanded: bool,
    pub active: bool,
}

impl<'a> StoryRow<'a> {
    pub fn new(story: &'a Story, expanded: bool, active: bool) -> Self {
        Self {
            story,
            expanded,
            active,
        }
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

        // Render "Description:" label
        if y < max_y {
            let line = Line::from("Description:").style(base_style);
            buf.set_line(area.x, y, &line, area.width);
            y += 1;
        }

        // Render description content or expansion hint
        if self.expanded {
            // Render each line of the description with indentation
            for desc_line in self.story.description.lines() {
                if y >= max_y {
                    break;
                }
                let line = Line::from(format!("  {}", desc_line)).style(base_style);
                buf.set_line(area.x, y, &line, area.width);
                y += 1;
            }
        } else {
            // Render expansion hint
            if y < max_y {
                let line = Line::from("Press <Space> to view description")
                .style(base_style.italic());
                buf.set_line(area.x, y, &line, area.width);
                y += 1;
            }
        }

        // Render empty line at the end
        if y < max_y {
            let line = Line::from("").style(base_style);
            buf.set_line(area.x, y, &line, area.width);
        }
    }

    fn height(&self) -> u16 {
        // Story name (1) + "Description:" label (1) + content + empty line (1)
        if self.expanded {
            let description_lines = self.story.description.lines().count() as u16;
            1 + 1 + description_lines + 1
        } else {
            // Story name (1) + "Description:" label (1) + hint (1) + empty line (1)
            4
        }
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
    fn test_height_collapsed() {
        let story = create_test_story("Test Story", "Single line description");
        let row = StoryRow::new(&story, false, false);
        assert_eq!(row.height(), 4);
    }

    #[test]
    fn test_height_expanded_single_line() {
        let story = create_test_story("Test Story", "Single line description");
        let row = StoryRow::new(&story, true, false);
        // 1 (name) + 1 (label) + 1 (description) + 1 (empty) = 4
        assert_eq!(row.height(), 4);
    }

    #[test]
    fn test_height_expanded_multi_line() {
        let story = create_test_story("Test Story", "Line 1\nLine 2\nLine 3");
        let row = StoryRow::new(&story, true, false);
        // 1 (name) + 1 (label) + 3 (description lines) + 1 (empty) = 6
        assert_eq!(row.height(), 6);
    }

    #[test]
    fn test_height_expanded_empty_description() {
        let story = create_test_story("Test Story", "");
        let row = StoryRow::new(&story, true, false);
        // 1 (name) + 1 (label) + 0 (no description lines) + 1 (empty) = 3
        assert_eq!(row.height(), 3);
    }
}
