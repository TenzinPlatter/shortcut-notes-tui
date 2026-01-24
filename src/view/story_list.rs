use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::border,
    text::Line,
    widgets::{Block, List, ListState, StatefulWidget, WidgetRef},
};

use crate::{
    api::story::Story,
    app::model::StoryListState,
};

/// Stateless view for story list
/// Holds references to data and state, rebuilt every frame
pub struct StoryListView<'a> {
    stories: &'a [Story],
    state: &'a StoryListState,
    is_focused: bool,
}

impl<'a> StoryListView<'a> {
    pub fn new(
        stories: &'a [Story],
        state: &'a StoryListState,
        is_focused: bool,
    ) -> Self {
        Self {
            stories,
            state,
            is_focused,
        }
    }
}

impl<'a> WidgetRef for StoryListView<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Build list items from stories + expansion state
        let list_items: Vec<_> = self
            .stories
            .iter()
            .enumerate()
            .map(|(idx, story)| {
                let is_expanded = self.state.expanded_items.contains(&idx);
                story.into_list_item(is_expanded)
            })
            .collect();

        let highlight_symbol = if self.is_focused { "> " } else { "  " };

        let list = List::new(list_items)
            .block(Block::bordered().border_set(border::THICK))
            .highlight_symbol(Line::from(highlight_symbol))
            .highlight_style(Style::default().blue().bold());

        // Create ephemeral ListState from our persistent state
        let mut list_state = ListState::default();
        list_state.select(self.state.selected_index);

        StatefulWidget::render(list, area, buf, &mut list_state);
    }
}
