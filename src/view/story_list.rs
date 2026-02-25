use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, StatefulWidget, Widget, WidgetRef},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::{
    api::{iteration::Iteration, story::Story},
    app::model::{LoadingState, StoryListState},
};

use super::story_item_builder::StoryItemWidget;

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Represents a group of stories belonging to the same iteration
pub struct IterationSection<'a> {
    pub iteration: Option<&'a Iteration>,
    pub stories: Vec<&'a Story>,
}

/// Groups stories by their iteration, sorted by iteration start date
fn group_stories_by_iteration<'a>(
    stories: &'a [Story],
    iterations: Option<&'a [Iteration]>,
    show_finished: bool,
) -> Vec<IterationSection<'a>> {
    // Build a HashMap grouping stories by iteration_id
    let mut grouped: HashMap<Option<i32>, Vec<&'a Story>> = HashMap::new();
    for story in stories {
        // Filter out completed stories if show_finished is false
        if !show_finished && story.completed {
            continue;
        }
        grouped.entry(story.iteration_id).or_default().push(story);
    }

    let mut sections = Vec::new();

    // If we have iterations, sort them by start_date and create sections
    if let Some(iterations) = iterations {
        let mut sorted_iterations: Vec<_> = iterations.iter().collect();
        sorted_iterations.sort_by_key(|it| it.start_date);

        for iteration in sorted_iterations {
            if let Some(mut stories) = grouped.remove(&Some(iteration.id)) {
                // Sort: unfinished first, then completed
                stories.sort_by_key(|s| s.completed);

                sections.push(IterationSection {
                    iteration: Some(iteration),
                    stories,
                });
            }
        }
    }

    // Add "No Iteration" section at the end if there are stories without an iteration
    if let Some(mut stories) = grouped.remove(&None) {
        stories.sort_by_key(|s| s.completed);

        sections.push(IterationSection {
            iteration: None,
            stories,
        });
    }

    sections
}

pub struct StoryListView<'a> {
    stories: &'a [Story],
    iterations: Option<&'a [Iteration]>,
    state: &'a StoryListState,
    active_story: Option<&'a Story>,
    is_focused: bool,
    loading: LoadingState,
    tick: usize,
}

impl<'a> StoryListView<'a> {
    pub fn new(
        stories: &'a [Story],
        iterations: Option<&'a [Iteration]>,
        state: &'a StoryListState,
        active_story: Option<&'a Story>,
        is_focused: bool,
        loading: LoadingState,
        tick: usize,
    ) -> Self {
        Self {
            stories,
            iterations,
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
        // Handle loading and empty states with a single bordered block
        if self.stories.is_empty() {
            let block = Block::bordered().border_set(border::THICK);
            let inner = block.inner(area);
            block.render(area, buf);

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
                let centered_area = Rect::new(inner.x, inner.y + inner.height / 2, inner.width, 1);
                paragraph.render(centered_area, buf);
            }
            return;
        }

        // Group stories by iteration
        let sections = group_stories_by_iteration(self.stories, self.iterations, self.state.show_finished);

        // Calculate layout constraints for sections:
        // header (1) + bordered list (items*2 + 2 for border) + spacing (1)
        let mut constraints = Vec::new();
        for section in &sections {
            constraints.push(Constraint::Length(1));
            constraints.push(Constraint::Length((section.stories.len() * 2 + 1) as u16));
            constraints.push(Constraint::Length(1));
        }

        if !constraints.is_empty() {
            constraints.pop();
        }
        constraints.push(Constraint::Min(0));

        let section_areas = Layout::vertical(constraints).split(area);

        let mut area_index = 0;
        for section in &sections {
            // Render section header
            let header_area = section_areas[area_index];
            area_index += 1;

            let header_text = if let Some(iteration) = section.iteration {
                iteration.name.clone()
            } else {
                "No Iteration".to_string()
            };

            let header_style = Style::default().dark_gray();
            let display = format!(" ── {} ──", header_text);
            let title_line = Line::from(display).style(header_style);
            buf.set_line(header_area.x, header_area.y, &title_line, header_area.width);

            // Render bordered stories list
            let list_area = section_areas[area_index];
            area_index += 1;

            let list_block = Block::bordered().border_set(border::THICK);
            let stories_area = list_block.inner(list_area);
            list_block.render(list_area, buf);

            let section_stories: Vec<_> = section.stories.to_vec();
            let active_story = self.active_story;
            let width = stories_area.width;

            let builder = ListBuilder::new(move |context| {
                let story = section_stories[context.index];
                let is_active = match active_story {
                    Some(active) => active.id == story.id,
                    None => false,
                };
                let is_completed = story.completed;
                let is_last = context.index == section_stories.len() - 1;

                let widget = StoryItemWidget::new(
                    story,
                    is_active,
                    context.is_selected,
                    width,
                    is_completed,
                    is_last,
                );
                let height = widget.height();

                (widget, height)
            });

            let list = ListView::new(builder, section.stories.len());

            let mut list_state = ListState::default();
            if let Some(selected_id) = self.state.selected_story_id
                && let Some(pos) = section.stories.iter().position(|s| s.id == selected_id)
            {
                list_state.select(Some(pos));
            }

            StatefulWidget::render(list, stories_area, buf, &mut list_state);

            // Skip spacing area
            area_index += 1;
        }
    }
}
