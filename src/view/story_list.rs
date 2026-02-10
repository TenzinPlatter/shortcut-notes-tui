use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, StatefulWidget, Widget, WidgetRef},
};

use crate::{api::story::Story, app::model::{LoadingState, StoryListState}};

use super::list::{CustomList, ListState};
use super::story_row::StoryRow;

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

        // Handle loading and empty states when no stories to display
        if self.stories.is_empty() {
            let message = if self.loading.is_loading() {
                // Show centered spinner + message
                format!("{} {}", self.spinner_char(), self.loading.label())
            } else {
                // Loaded but no stories
                "No stories assigned in this iteration.".to_string()
            };

            let style = Style::default().gray();
            let paragraph = Paragraph::new(message)
                .style(style)
                .alignment(Alignment::Center);

            // Center vertically
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

        // Convert stories to StoryRow instances
        let story_rows: Vec<_> = self
            .stories
            .iter()
            .map(|story| {
                let is_active = match self.active_story {
                    Some(active_story) => active_story.id == story.id,
                    None => false,
                };

                StoryRow::new(story, is_active)
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

        custom_list.render(inner, buf, &mut list_state);
    }
}
