use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{
    api::iteration::Iteration,
    app::{cmd::Cmd, model::IterationListState, msg::IterationListMsg},
    navkey,
};

/// Returns the subset of `items` whose name fuzzy-matches `query`.
/// If `query` is empty, all items are returned.
pub fn filter_items<'a>(items: &'a [Iteration], query: &str) -> Vec<&'a Iteration> {
    if query.is_empty() {
        return items.iter().collect();
    }
    let matcher = SkimMatcherV2::default();
    items
        .iter()
        .filter(|it| matcher.fuzzy_match(&it.name, query).is_some())
        .collect()
}

/// Returns active iterations first, then all others (excluding actives), both filtered by query.
pub fn combined_visible<'a>(
    current: &'a [Iteration],
    all: &'a [Iteration],
    query: &str,
) -> Vec<&'a Iteration> {
    let active_filtered = filter_items(current, query);
    let active_ids: HashSet<i32> = active_filtered.iter().map(|it| it.id).collect();
    let rest_filtered: Vec<&Iteration> = filter_items(all, query)
        .into_iter()
        .filter(|it| !active_ids.contains(&it.id))
        .collect();
    active_filtered.into_iter().chain(rest_filtered).collect()
}

pub fn update(
    state: &mut IterationListState,
    current_iterations: &[Iteration],
    all_iterations: &[Iteration],
    msg: IterationListMsg,
) -> Vec<Cmd> {
    match msg {
        IterationListMsg::FocusNext => {
            let visible = combined_visible(
                current_iterations,
                all_iterations,
                &state.search_query.clone(),
            );
            if visible.is_empty() {
                return vec![Cmd::None];
            }

            let current_idx = state
                .selected_id
                .and_then(|id| visible.iter().position(|it| it.id == id));

            let next_idx = match current_idx {
                Some(idx) => (idx + 1) % visible.len(),
                None => 0,
            };

            state.selected_id = Some(visible[next_idx].id);
            vec![Cmd::None]
        }

        IterationListMsg::FocusPrev => {
            let visible = combined_visible(
                current_iterations,
                all_iterations,
                &state.search_query.clone(),
            );
            if visible.is_empty() {
                return vec![Cmd::None];
            }

            let current_idx = state
                .selected_id
                .and_then(|id| visible.iter().position(|it| it.id == id));

            let prev_idx = match current_idx {
                Some(0) => visible.len() - 1,
                Some(idx) => idx - 1,
                None => visible.len() - 1,
            };

            state.selected_id = Some(visible[prev_idx].id);
            vec![Cmd::None]
        }

        IterationListMsg::OpenNote => {
            let selected = state.selected_id.and_then(|id| {
                current_iterations
                    .iter()
                    .find(|it| it.id == id)
                    .or_else(|| all_iterations.iter().find(|it| it.id == id))
            });

            if let Some(iteration) = selected {
                vec![Cmd::OpenIterationNote {
                    iteration_id: iteration.id,
                    iteration_name: iteration.name.clone(),
                    iteration_app_url: iteration.app_url.clone(),
                }]
            } else {
                vec![Cmd::None]
            }
        }

        IterationListMsg::ActivateSearch => {
            state.search_active = true;
            vec![Cmd::None]
        }

        IterationListMsg::DeactivateSearch => {
            state.search_active = false;
            vec![Cmd::None]
        }

        IterationListMsg::SearchInput(c) => {
            state.search_query.push(c);
            // Reset selection to first visible item after query changes
            let visible = combined_visible(
                current_iterations,
                all_iterations,
                &state.search_query.clone(),
            );
            state.selected_id = visible.first().map(|it| it.id);
            vec![Cmd::None]
        }

        IterationListMsg::SearchBackspace => {
            state.search_query.pop();
            let visible = combined_visible(
                current_iterations,
                all_iterations,
                &state.search_query.clone(),
            );
            if state.selected_id.is_none() {
                state.selected_id = visible.first().map(|it| it.id);
            }
            vec![Cmd::None]
        }

        IterationListMsg::ClearSearch => {
            state.search_query.clear();
            state.search_active = false;
            vec![Cmd::None]
        }
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<IterationListMsg> {
    match key.code {
        navkey!(down) => Some(IterationListMsg::FocusNext),
        navkey!(up) => Some(IterationListMsg::FocusPrev),
        KeyCode::Enter => Some(IterationListMsg::OpenNote),
        _ => None,
    }
}
