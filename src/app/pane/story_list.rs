use crossterm::event::KeyEvent;

use crate::{
    api::{iteration::Iteration, story::Story},
    app::{cmd::Cmd, msg::StoryListMsg},
    keys::AppKey,
};

pub use crate::app::model::StoryListState;

pub fn update(
    state: &mut StoryListState,
    stories: &[Story],
    current_iteration: Option<&Iteration>,
    msg: StoryListMsg,
) -> Vec<Cmd> {
    match msg {
        StoryListMsg::SelectNext => {
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

        StoryListMsg::SelectPrev => {
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
            if let Some(idx) = state.selected_index
                && let Some(story) = stories.get(idx)
            {
                return vec![Cmd::OpenNote {
                    story: story.clone(),
                    iteration: current_iteration.cloned(),
                }];
            }
            vec![Cmd::None]
        }
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<StoryListMsg> {
    match key.code.try_into() {
        Ok(AppKey::Down) => Some(StoryListMsg::SelectNext),
        Ok(AppKey::Up) => Some(StoryListMsg::SelectPrev),
        Ok(AppKey::Select) => Some(StoryListMsg::ToggleExpand),
        Ok(AppKey::Edit) => Some(StoryListMsg::OpenNote),
        _ => None,
    }
}
