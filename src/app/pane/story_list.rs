use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    api::{
        iteration::Iteration,
        story::{Story, get_story_associated_iteration},
    },
    app::{cmd::Cmd, msg::StoryListMsg},
    dbg_file, navkey,
};

pub use crate::app::model::StoryListState;

/// Represents a group of stories belonging to the same iteration
struct IterationSection<'a> {
    #[allow(dead_code)]
    iteration: Option<&'a Iteration>,
    stories: Vec<&'a Story>,
}

/// Groups stories by their iteration, sorted by iteration start date
fn group_stories_by_iteration<'a>(
    stories: &'a [Story],
    iterations: Option<&[&'a Iteration]>,
) -> Vec<IterationSection<'a>> {
    // Build a HashMap grouping stories by iteration_id
    let mut grouped: HashMap<Option<i32>, Vec<&'a Story>> = HashMap::new();
    for story in stories {
        grouped.entry(story.iteration_id).or_default().push(story);
    }

    let mut sections = Vec::new();

    // If we have iterations, sort them by start_date and create sections
    if let Some(iterations) = iterations {
        let mut sorted_iterations: Vec<_> = iterations.to_vec();
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

/// Find the position of a story within the grouped sections
/// Returns (section_index, story_index_in_section)
fn find_story_position(story_id: i32, sections: &[IterationSection]) -> Option<(usize, usize)> {
    for (section_idx, section) in sections.iter().enumerate() {
        if let Some(story_idx) = section.stories.iter().position(|s| s.id == story_id) {
            return Some((section_idx, story_idx));
        }
    }
    None
}

/// Get the next story ID when navigating down
fn next_story_id(current_story_id: i32, sections: &[IterationSection]) -> Option<i32> {
    if sections.is_empty() {
        return None;
    }

    let (section_idx, story_idx) = find_story_position(current_story_id, sections)?;

    // Try next story in same section
    if story_idx + 1 < sections[section_idx].stories.len() {
        return Some(sections[section_idx].stories[story_idx + 1].id);
    }

    // Try first story of next section
    if section_idx + 1 < sections.len() {
        return sections[section_idx + 1].stories.first().map(|s| s.id);
    }

    // Wrap to first story of first section
    sections.first()?.stories.first().map(|s| s.id)
}

/// Get the previous story ID when navigating up
fn prev_story_id(current_story_id: i32, sections: &[IterationSection]) -> Option<i32> {
    if sections.is_empty() {
        return None;
    }

    let (section_idx, story_idx) = find_story_position(current_story_id, sections)?;

    // Try previous story in same section
    if story_idx > 0 {
        return Some(sections[section_idx].stories[story_idx - 1].id);
    }

    // Try last story of previous section
    if section_idx > 0 {
        return sections[section_idx - 1].stories.last().map(|s| s.id);
    }

    // Wrap to last story of last section
    sections.last()?.stories.last().map(|s| s.id)
}

pub fn update(
    state: &mut StoryListState,
    stories: &[Story],
    current_iterations: Option<Vec<&Iteration>>,
    msg: StoryListMsg,
) -> Vec<Cmd> {
    match msg {
        StoryListMsg::FocusNext => {
            if stories.is_empty() {
                return vec![Cmd::None];
            }

            // Group stories to handle section boundaries
            let sections = group_stories_by_iteration(stories, current_iterations.as_deref());

            if let Some(current_id) = state.selected_story_id {
                state.selected_story_id = next_story_id(current_id, &sections);
            } else {
                // No story selected, select first story in first section
                state.selected_story_id = sections
                    .first()
                    .and_then(|s| s.stories.first())
                    .map(|s| s.id);
            }

            vec![Cmd::None]
        }

        StoryListMsg::FocusPrev => {
            if stories.is_empty() {
                return vec![Cmd::None];
            }

            // Group stories to handle section boundaries
            let sections = group_stories_by_iteration(stories, current_iterations.as_deref());

            if let Some(current_id) = state.selected_story_id {
                state.selected_story_id = prev_story_id(current_id, &sections);
            } else {
                // No story selected, select last story in last section
                state.selected_story_id =
                    sections.last().and_then(|s| s.stories.last()).map(|s| s.id);
            }

            vec![Cmd::None]
        }

        StoryListMsg::OpenNote => {
            if let Some(story) = get_hovered_story(state, stories) {
                let iteration_app_url = current_iterations
                    .and_then(|iterations| {
                        get_story_associated_iteration(story.iteration_id, iterations)
                    })
                    .map(|it| it.app_url.clone());

                return vec![Cmd::OpenNote {
                    story_id: story.id,
                    story_name: story.name.clone(),
                    story_app_url: story.app_url.clone(),
                    iteration_app_url,
                }];
            }
            vec![Cmd::None]
        }

        StoryListMsg::SelectStory => {
            let story = get_hovered_story(state, stories);
            dbg_file!("Setting story: {:?} to active", story);
            vec![Cmd::SelectStory(story), Cmd::WriteCache]
        }

        StoryListMsg::TmuxEnter => {
            if let Some(story) = get_hovered_story(state, stories) {
                vec![Cmd::OpenTmuxSession {
                    story_name: story.name.clone(),
                }]
            } else {
                vec![Cmd::None]
            }
        }

        StoryListMsg::EditStoryContents => {
            if let Some(story) = get_hovered_story(state, stories) {
                vec![Cmd::EditStoryContent {
                    story_id: story.id,
                    description: story.description.clone(),
                }]
            } else {
                vec![Cmd::None]
            }
        }

        StoryListMsg::OpenInBrowser => {
            if let Some(story) = get_hovered_story(state, stories) {
                vec![Cmd::OpenInBrowser {
                    app_url: story.app_url.clone(),
                }]
            } else {
                vec![Cmd::None]
            }
        }
    }
}

fn get_hovered_story(state: &StoryListState, stories: &[Story]) -> Option<Story> {
    let id = state.selected_story_id?;
    stories.iter().find(|s| s.id == id).cloned()
}

pub fn key_to_msg(key: KeyEvent) -> Option<StoryListMsg> {
    match key.code {
        navkey!(down) => Some(StoryListMsg::FocusNext),
        navkey!(up) => Some(StoryListMsg::FocusPrev),
        KeyCode::Char('o') => Some(StoryListMsg::OpenInBrowser),
        KeyCode::Char('n') => Some(StoryListMsg::OpenNote),
        _ => None,
    }
}
