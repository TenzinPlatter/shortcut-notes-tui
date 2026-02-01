use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::border,
    text::Line,
    widgets::{Block, StatefulWidget, Widget, WidgetRef},
};

use crate::{api::story::Story, app::model::StoryListState};

use super::list::{CustomList, ListState};
use super::story_row::StoryRow;

pub struct StoryListView<'a> {
    stories: &'a [Story],
    state: &'a StoryListState,
    is_focused: bool,
}

impl<'a> StoryListView<'a> {
    pub fn new(stories: &'a [Story], state: &'a StoryListState, is_focused: bool) -> Self {
        Self {
            stories,
            state,
            is_focused,
        }
    }
}

impl<'a> WidgetRef for StoryListView<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Create the border block
        let block = Block::bordered().border_set(border::THICK);
        let inner = block.inner(area);
        block.render(area, buf);

        // Convert stories to StoryRow instances
        let story_rows: Vec<_> = self
            .stories
            .iter()
            .enumerate()
            .map(|(idx, story)| {
                let is_expanded = self.state.expanded_items.contains(&idx);
                let is_active = match &self.state.active_story {
                    Some(active_story) => active_story.id == story.id,
                    None => false,
                };

                StoryRow::new(story, is_expanded, is_active)
            })
            .collect();

        // Create horizontal divider line (repeated "─" character)
        let divider = Line::from("─".repeat(inner.width as usize));

        // Determine highlight style based on focus
        let highlight_style = if self.is_focused {
            Style::default().blue().bold()
        } else {
            Style::default()
        };

        // Create and render the CustomList
        let custom_list = CustomList::new(&story_rows)
            .divider(divider)
            .highlight_style(highlight_style);

        let mut list_state = ListState::default();
        list_state.select(self.state.selected_index);

        StatefulWidget::render(custom_list, inner, buf, &mut list_state);
    }
}
