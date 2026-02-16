use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    symbols::border,
    widgets::{Block, Paragraph, StatefulWidget, Widget, WidgetRef},
};
use tui_widget_list::{ListBuilder, ListView, ListState};

use crate::{
    api::story::Story,
    app::model::{LoadingState, StoryListState},
};

use super::story_item_builder::StoryItemWidget;

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

pub struct StoryListView<'a> {
    stories: &'a [Story],
    state: &'a StoryListState,
    active_story: Option<&'a Story>,
    is_focused: bool,
    loading: LoadingState,
    tick: usize,
}

impl<'a> StoryListView<'a> {
    pub fn new(
        stories: &'a [Story],
        state: &'a StoryListState,
        active_story: Option<&'a Story>,
        is_focused: bool,
        loading: LoadingState,
        tick: usize,
    ) -> Self {
        Self {
            stories,
            state,
            active_story,
            is_focused,
            loading,
            tick,
        }
    }

    fn spinner_char(&self) -> char {
        SPINNER_CHARS[self.tick % SPINNER_CHARS.len()]
    }
}

impl<'a> WidgetRef for StoryListView<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Create the border block
        let block = Block::bordered().border_set(border::THICK);
        let inner = block.inner(area);
        block.render(area, buf);

        // Handle loading and empty states
        if self.stories.is_empty() {
            let message = if self.loading.is_loading() {
                format!("{} {}", self.spinner_char(), self.loading.label())
            } else {
                "No stories assigned in this iteration.".to_string()
            };

            let style = Style::default().gray();
            let paragraph = Paragraph::new(message)
                .style(style)
                .alignment(Alignment::Center);

            if inner.height > 0 {
                let centered_area = Rect::new(
                    inner.x,
                    inner.y + inner.height / 2,
                    inner.width,
                    1,
                );
                paragraph.render(centered_area, buf);
            }
            return;
        }

        // Determine highlight style based on focus
        let highlight_style = if self.is_focused {
            Style::default().blue().bold()
        } else {
            Style::default()
        };

        // Create the list builder
        let stories = self.stories;
        let active_story = self.active_story;
        let width = inner.width;

        let builder = ListBuilder::new(move |context| {
            let story = &stories[context.index];
            let is_active = match active_story {
                Some(active) => active.id == story.id,
                None => false,
            };

            let widget = StoryItemWidget::new(story, is_active, context.is_selected, highlight_style, width);
            let height = widget.height();

            (widget, height)
        });

        // Create the ListView
        let list = ListView::new(builder, self.stories.len());

        // Create mutable state from our StoryListState
        let mut list_state = ListState::default();
        if let Some(selected) = self.state.selected_index(self.stories) {
            list_state.select(Some(selected));
        }

        // Render the list
        StatefulWidget::render(list, inner, buf, &mut list_state);
    }
}
