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
) -> Vec<IterationSection<'a>> {
    // Build a HashMap grouping stories by iteration_id
    let mut grouped: HashMap<Option<i32>, Vec<&'a Story>> = HashMap::new();
    for story in stories {
        grouped.entry(story.iteration_id).or_default().push(story);
    }

    let mut sections = Vec::new();

    // If we have iterations, sort them by start_date and create sections
    if let Some(iterations) = iterations {
        let mut sorted_iterations: Vec<_> = iterations.iter().collect();
        sorted_iterations.sort_by_key(|it| it.start_date);

        for iteration in sorted_iterations {
            if let Some(stories) = grouped.remove(&Some(iteration.id)) {
                sections.push(IterationSection {
                    iteration: Some(iteration),
                    stories,
                });
            }
        }
    }

    // Add "No Iteration" section at the end if there are stories without an iteration
    if let Some(stories) = grouped.remove(&None) {
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
                let centered_area = Rect::new(inner.x, inner.y + inner.height / 2, inner.width, 1);
                paragraph.render(centered_area, buf);
            }
            return;
        }

        // Group stories by iteration
        let sections = group_stories_by_iteration(self.stories, self.iterations);

        // Determine highlight style based on focus
        let highlight_style = if self.is_focused {
            Style::default().blue().bold()
        } else {
            Style::default()
        };

        // Calculate layout constraints for sections
        let mut constraints = Vec::new();
        for section in &sections {
            // Header: 2 lines (title + divider)
            constraints.push(Constraint::Length(2));
            // Stories: 2 lines each
            constraints.push(Constraint::Length((section.stories.len() * 2) as u16));
            // Spacing: 1 line between sections
            constraints.push(Constraint::Length(1));
        }

        // If we have constraints, remove the last spacing constraint
        if !constraints.is_empty() {
            constraints.pop();
        }

        // Add a filler constraint to take up remaining space
        constraints.push(Constraint::Min(0));

        let section_areas = Layout::vertical(constraints).split(inner);

        // Render each section
        let mut area_index = 0;
        for section in &sections {
            // Render header
            let header_area = section_areas[area_index];
            area_index += 1;

            let header_text = if let Some(iteration) = section.iteration {
                iteration.name.clone()
            } else {
                "No Iteration".to_string()
            };

            let header_style = if section.iteration.is_some() {
                Style::default().cyan().bold()
            } else {
                Style::default().gray().bold()
            };

            // Render title line
            if header_area.height > 0 {
                let title_line = Line::from(header_text).style(header_style);
                buf.set_line(header_area.x, header_area.y, &title_line, header_area.width);
            }

            // Render divider line
            if header_area.height > 1 {
                let divider = "═".repeat(header_area.width as usize);
                let divider_line = Line::from(divider).style(header_style);
                buf.set_line(
                    header_area.x,
                    header_area.y + 1,
                    &divider_line,
                    header_area.width,
                );
            }

            // Render stories list
            let stories_area = section_areas[area_index];
            area_index += 1;

            // Build a list of stories for this section
            let section_stories: Vec<_> = section.stories.to_vec();
            let active_story = self.active_story;
            let width = stories_area.width;

            let builder = ListBuilder::new(move |context| {
                let story = section_stories[context.index];
                let is_active = match active_story {
                    Some(active) => active.id == story.id,
                    None => false,
                };

                let widget = StoryItemWidget::new(
                    story,
                    is_active,
                    context.is_selected,
                    highlight_style,
                    width,
                );
                let height = widget.height();

                (widget, height)
            });

            let list = ListView::new(builder, section.stories.len());

            // Find if any story in this section is selected
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
