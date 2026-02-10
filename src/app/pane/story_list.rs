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

            let current_idx = state.selected_index(stories).unwrap_or(0);
            let next_idx = if current_idx >= stories.len() - 1 {
                0 // Wrap around
            } else {
                current_idx + 1
            };

            state.selected_story_id = stories.get(next_idx).map(|s| s.id);
            vec![Cmd::None]
        }

        StoryListMsg::FocusPrev => {
            if stories.is_empty() {
                return vec![Cmd::None];
            }

            let current_idx = state.selected_index(stories).unwrap_or(0);
            let prev_idx = if current_idx == 0 {
                stories.len() - 1 // Wrap around
            } else {
                current_idx - 1
            };

            state.selected_story_id = stories.get(prev_idx).map(|s| s.id);
            vec![Cmd::None]
        }

        StoryListMsg::OpenNote => {
            if let Some(story) = get_selected_story(state, stories) {
                let iteration_app_url = current_iterations
                    .and_then(|iterations| get_story_associated_iteration(story.iteration_id, iterations))
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
            let story = get_selected_story(state, stories);
            dbg_file!("Setting story: {:?} to active", story);
            vec![Cmd::SelectStory(story), Cmd::WriteCache]
        }

        StoryListMsg::TmuxEnter => {
            if let Some(story) = get_selected_story(state, stories) {
                vec![Cmd::OpenTmuxSession {
                    story_name: story.name.clone(),
                }]
            } else {
                vec![Cmd::None]
            }
        }

        StoryListMsg::EditStoryContents => {
            if let Some(story) = get_selected_story(state, stories) {
                vec![Cmd::EditStoryContent {
                    story_id: story.id,
                    description: story.description.clone(),
                }]
            } else {
                vec![Cmd::None]
            }
        }
    }
}

fn get_selected_story(state: &StoryListState, stories: &[Story]) -> Option<Story> {
    let id = state.selected_story_id?;
    stories.iter().find(|s| s.id == id).cloned()
}

pub fn key_to_msg(key: KeyEvent) -> Option<StoryListMsg> {
    match key.code {
        navkey!(down) => Some(StoryListMsg::FocusNext),
        navkey!(up) => Some(StoryListMsg::FocusPrev),
        _ => None,
    }
}
