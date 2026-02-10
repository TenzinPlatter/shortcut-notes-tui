use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    api::story::Story,
    app::{cmd::Cmd, model::DescriptionModalState, msg::DescriptionModalMsg},
    navkey,
};

pub fn update(
    state: &mut DescriptionModalState,
    msg: DescriptionModalMsg,
    visible_height: u16,
    total_lines: u16,
) -> Vec<Cmd> {
    let max_scroll = total_lines.saturating_sub(visible_height);

    match msg {
        DescriptionModalMsg::Open => {
            // Handled in main update - this shouldn't be called directly
            vec![Cmd::None]
        }

        DescriptionModalMsg::Close => {
            state.is_showing = false;
            state.scroll_offset = 0;
            state.story = None;
            vec![Cmd::None]
        }

        DescriptionModalMsg::ScrollUp => {
            state.scroll_offset = state.scroll_offset.saturating_sub(1);
            vec![Cmd::None]
        }

        DescriptionModalMsg::ScrollDown => {
            state.scroll_offset = (state.scroll_offset + 1).min(max_scroll);
            vec![Cmd::None]
        }

        DescriptionModalMsg::ScrollHalfPageUp => {
            let half_page = visible_height / 2;
            state.scroll_offset = state.scroll_offset.saturating_sub(half_page);
            vec![Cmd::None]
        }

        DescriptionModalMsg::ScrollHalfPageDown => {
            let half_page = visible_height / 2;
            state.scroll_offset = (state.scroll_offset + half_page).min(max_scroll);
            vec![Cmd::None]
        }

        DescriptionModalMsg::ScrollToTop => {
            state.scroll_offset = 0;
            vec![Cmd::None]
        }

        DescriptionModalMsg::ScrollToBottom => {
            state.scroll_offset = max_scroll;
            vec![Cmd::None]
        }
    }
}

pub fn open(state: &mut DescriptionModalState, story: Story) {
    state.is_showing = true;
    state.scroll_offset = 0;
    state.story = Some(story);
}

pub fn key_to_msg(key: KeyEvent) -> Option<DescriptionModalMsg> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Some(DescriptionModalMsg::Close),
        navkey!(down) => Some(DescriptionModalMsg::ScrollDown),
        navkey!(up) => Some(DescriptionModalMsg::ScrollUp),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(DescriptionModalMsg::ScrollHalfPageDown)
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(DescriptionModalMsg::ScrollHalfPageUp)
        }
        KeyCode::PageDown => Some(DescriptionModalMsg::ScrollHalfPageDown),
        KeyCode::PageUp => Some(DescriptionModalMsg::ScrollHalfPageUp),
        KeyCode::Char('g') => Some(DescriptionModalMsg::ScrollToTop),
        KeyCode::Char('G') => Some(DescriptionModalMsg::ScrollToBottom),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn create_test_story() -> Story {
        Story {
            id: 1,
            name: "Test Story".to_string(),
            description: "Test description".to_string(),
            completed: false,
            branches: vec![],
            comments: vec![],
            epic_id: None,
            iteration_id: None,
            app_url: "https://example.com".to_string(),
        }
    }

    #[test]
    fn test_open_sets_state() {
        let mut state = DescriptionModalState::default();
        let story = create_test_story();

        open(&mut state, story.clone());

        assert!(state.is_showing);
        assert_eq!(state.scroll_offset, 0);
        assert_eq!(state.story.as_ref().unwrap().id, story.id);
    }

    #[test]
    fn test_close_resets_state() {
        let mut state = DescriptionModalState {
            is_showing: true,
            scroll_offset: 10,
            story: Some(create_test_story()),
        };

        update(&mut state, DescriptionModalMsg::Close, 20, 50);

        assert!(!state.is_showing);
        assert_eq!(state.scroll_offset, 0);
        assert!(state.story.is_none());
    }

    #[test]
    fn test_scroll_down_increments() {
        let mut state = DescriptionModalState {
            is_showing: true,
            scroll_offset: 0,
            story: Some(create_test_story()),
        };

        update(&mut state, DescriptionModalMsg::ScrollDown, 20, 50);

        assert_eq!(state.scroll_offset, 1);
    }

    #[test]
    fn test_scroll_down_clamps_to_max() {
        let mut state = DescriptionModalState {
            is_showing: true,
            scroll_offset: 30, // max_scroll = 50 - 20 = 30
            story: Some(create_test_story()),
        };

        update(&mut state, DescriptionModalMsg::ScrollDown, 20, 50);

        assert_eq!(state.scroll_offset, 30); // Should not exceed max
    }

    #[test]
    fn test_scroll_up_decrements() {
        let mut state = DescriptionModalState {
            is_showing: true,
            scroll_offset: 5,
            story: Some(create_test_story()),
        };

        update(&mut state, DescriptionModalMsg::ScrollUp, 20, 50);

        assert_eq!(state.scroll_offset, 4);
    }

    #[test]
    fn test_scroll_up_clamps_to_zero() {
        let mut state = DescriptionModalState {
            is_showing: true,
            scroll_offset: 0,
            story: Some(create_test_story()),
        };

        update(&mut state, DescriptionModalMsg::ScrollUp, 20, 50);

        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut state = DescriptionModalState {
            is_showing: true,
            scroll_offset: 25,
            story: Some(create_test_story()),
        };

        update(&mut state, DescriptionModalMsg::ScrollToTop, 20, 50);

        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut state = DescriptionModalState {
            is_showing: true,
            scroll_offset: 0,
            story: Some(create_test_story()),
        };

        update(&mut state, DescriptionModalMsg::ScrollToBottom, 20, 50);

        assert_eq!(state.scroll_offset, 30); // max_scroll = 50 - 20
    }

    #[test]
    fn test_key_to_msg_mappings() {
        let make_key = |code: KeyCode, modifiers: KeyModifiers| KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };

        assert!(matches!(
            key_to_msg(make_key(KeyCode::Esc, KeyModifiers::NONE)),
            Some(DescriptionModalMsg::Close)
        ));

        assert!(matches!(
            key_to_msg(make_key(KeyCode::Char('q'), KeyModifiers::NONE)),
            Some(DescriptionModalMsg::Close)
        ));

        assert!(matches!(
            key_to_msg(make_key(KeyCode::Char('j'), KeyModifiers::NONE)),
            Some(DescriptionModalMsg::ScrollDown)
        ));

        assert!(matches!(
            key_to_msg(make_key(KeyCode::Char('k'), KeyModifiers::NONE)),
            Some(DescriptionModalMsg::ScrollUp)
        ));

        assert!(matches!(
            key_to_msg(make_key(KeyCode::Char('g'), KeyModifiers::NONE)),
            Some(DescriptionModalMsg::ScrollToTop)
        ));

        assert!(matches!(
            key_to_msg(make_key(KeyCode::Char('G'), KeyModifiers::NONE)),
            Some(DescriptionModalMsg::ScrollToBottom)
        ));
    }
}
