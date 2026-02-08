use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    api::{iteration::Iteration, story::Story},
    app::{cmd::Cmd, msg::StoryListMsg},
    dbg_file, navkey,
};

pub use crate::app::model::StoryListState;

pub fn update(
    state: &mut StoryListState,
    stories: &[Story],
    current_iteration: Option<&Iteration>,
    msg: StoryListMsg,
) -> Vec<Cmd> {
    match msg {
        StoryListMsg::FocusNext => {
            if stories.is_empty() {
                return vec![Cmd::None];
            }

            state.selected_index = Some(match state.selected_index {
                None => 0,
                Some(idx) if idx >= stories.len() - 1 => 0, // Wrap around
                Some(idx) => idx + 1,
            });

            vec![Cmd::None]
        }

        StoryListMsg::FocusPrev => {
            if stories.is_empty() {
                return vec![Cmd::None];
            }

            state.selected_index = Some(match state.selected_index {
                None => 0,
                Some(0) => stories.len() - 1, // Wrap around
                Some(idx) => idx - 1,
            });

            vec![Cmd::None]
        }

        StoryListMsg::ToggleExpand => {
            if let Some(idx) = state.selected_index {
                if state.expanded_items.contains(&idx) {
                    state.expanded_items.remove(&idx);
                } else {
                    state.expanded_items.insert(idx);
                }
            }
            vec![Cmd::None]
        }

        StoryListMsg::OpenNote => {
            if let Some(story) = get_selected_story(state, stories) {
                return vec![Cmd::OpenNote {
                    story: story.clone(),
                    iteration: current_iteration.cloned(),
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
                vec![Cmd::EditStoryContent(story.clone())]
            } else {
                vec![Cmd::None]
            }
        }
    }
}

fn get_selected_story(state: &StoryListState, stories: &[Story]) -> Option<Story> {
    if let Some(idx) = state.selected_index {
        stories.get(idx).cloned()
    } else {
        None
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<StoryListMsg> {
    match key.code {
        navkey!(down) => Some(StoryListMsg::FocusNext),
        navkey!(up) => Some(StoryListMsg::FocusPrev),
        KeyCode::Char(' ') => Some(StoryListMsg::ToggleExpand),
        _ => None,
    }
}
