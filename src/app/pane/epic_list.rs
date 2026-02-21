use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use crate::{
    api::epic::EpicSlim,
    app::{cmd::Cmd, model::EpicListState, msg::EpicListMsg},
    navkey,
};

/// Returns the subset of `items` whose name fuzzy-matches `query`.
/// If `query` is empty, all items are returned.
pub fn filter_items<'a>(items: &'a [EpicSlim], query: &str) -> Vec<&'a EpicSlim> {
    if query.is_empty() {
        return items.iter().collect();
    }
    let matcher = SkimMatcherV2::default();
    items
        .iter()
        .filter(|e| matcher.fuzzy_match(&e.name, query).is_some())
        .collect()
}

pub fn update(state: &mut EpicListState, epics: &[EpicSlim], msg: EpicListMsg) -> Vec<Cmd> {
    match msg {
        EpicListMsg::FocusNext => {
            let visible = filter_items(epics, &state.search_query.clone());
            if visible.is_empty() {
                return vec![Cmd::None];
            }

            let current_idx = state
                .selected_id
                .and_then(|id| visible.iter().position(|e| e.id == id));

            let next_idx = match current_idx {
                Some(idx) => (idx + 1) % visible.len(),
                None => 0,
            };

            state.selected_id = Some(visible[next_idx].id);
            vec![Cmd::None]
        }

        EpicListMsg::FocusPrev => {
            let visible = filter_items(epics, &state.search_query.clone());
            if visible.is_empty() {
                return vec![Cmd::None];
            }

            let current_idx = state
                .selected_id
                .and_then(|id| visible.iter().position(|e| e.id == id));

            let prev_idx = match current_idx {
                Some(0) => visible.len() - 1,
                Some(idx) => idx - 1,
                None => visible.len() - 1,
            };

            state.selected_id = Some(visible[prev_idx].id);
            vec![Cmd::None]
        }

        EpicListMsg::OpenNote => {
            let selected = state
                .selected_id
                .and_then(|id| epics.iter().find(|e| e.id == id));

            if let Some(epic) = selected {
                vec![Cmd::OpenEpicNote {
                    epic_id: epic.id,
                    epic_name: epic.name.clone(),
                    epic_app_url: epic.app_url.clone(),
                }]
            } else {
                vec![Cmd::None]
            }
        }

        EpicListMsg::ActivateSearch => {
            state.search_active = true;
            vec![Cmd::None]
        }

        EpicListMsg::DeactivateSearch => {
            state.search_active = false;
            vec![Cmd::None]
        }

        EpicListMsg::SearchInput(c) => {
            state.search_query.push(c);
            let visible = filter_items(epics, &state.search_query.clone());
            state.selected_id = visible.first().map(|e| e.id);
            vec![Cmd::None]
        }

        EpicListMsg::SearchBackspace => {
            state.search_query.pop();
            let visible = filter_items(epics, &state.search_query.clone());
            if state.selected_id.is_none() {
                state.selected_id = visible.first().map(|e| e.id);
            }
            vec![Cmd::None]
        }

        EpicListMsg::ClearSearch => {
            state.search_query.clear();
            state.search_active = false;
            vec![Cmd::None]
        }
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<EpicListMsg> {
    match key.code {
        navkey!(down) => Some(EpicListMsg::FocusNext),
        navkey!(up) => Some(EpicListMsg::FocusPrev),
        KeyCode::Enter => Some(EpicListMsg::OpenNote),
        _ => None,
    }
}
