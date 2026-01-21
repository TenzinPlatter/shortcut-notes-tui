use std::{cell::Cell, collections::HashSet};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::border,
    text::Line,
    widgets::{Block, List, ListState, StatefulWidget, WidgetRef},
};

use crate::{
    keys::{AppKey, KeyHandler},
    pane::Selectable,
    view::list::ExpandableListItem,
};

pub struct ListPane<T: ExpandableListItem + Clone> {
    list: List<'static>,
    list_items_shadow: Vec<T>,
    state: Cell<ListState>,
    expanded_item_indexes: HashSet<usize>,
    is_selected: bool,
}

impl<T: ExpandableListItem + Clone> ListPane<T> {
    pub fn new(items: Vec<T>) -> Self {
        let list_items: Vec<_> = items
            .clone()
            .into_iter()
            .map(|s| s.as_list_item(false))
            .collect();

        let block = Block::bordered().border_set(border::THICK);
        let highlighted_symbol: Line = "> ".into();

        let list = List::new(list_items)
            .block(block)
            .highlight_symbol(highlighted_symbol)
            .highlight_style(Style::default().blue().bold());

        let state = Cell::new(ListState::default());

        ListPane {
            list,
            state,
            list_items_shadow: items,
            is_selected: false,
            expanded_item_indexes: HashSet::new(),
        }
    }

    pub fn with_state(mut self, state: ListState) -> Self {
        self.state = Cell::new(state);
        self
    }
}

impl<T: ExpandableListItem + Clone> WidgetRef for ListPane<T> {
    #[doc = " Draws the current state of the widget in the given buffer. That is the only method required"]
    #[doc = " to implement a custom widget."]
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.get();
        (&self.list).render(area, buf, &mut state);
        self.state.set(state);
    }
}

impl<T: ExpandableListItem + Clone> Selectable for ListPane<T> {
    fn is_selected(&self) -> bool {
        self.is_selected
    }

    fn select(&mut self) {
        self.is_selected = true;
        if self.state.get().selected().is_none() {
            self.state.get_mut().select(Some(0));
        }
    }

    fn unselect(&mut self) {
        self.is_selected = false;
        if self.state.get().selected().is_some() {
            self.state.get_mut().select(None);
        }
    }
}

impl<T: ExpandableListItem + Clone> KeyHandler for ListPane<T> {
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> bool {
        match key_event.code.try_into() {
            Ok(AppKey::Up) => {
                if let Some(curr) = self.state.get().selected() {
                    if curr != 0 {
                        self.state.get_mut().select(Some(curr - 1));
                    } else {
                        let len = self.list.len();
                        self.state.get_mut().select(Some(len - 1));
                    }
                } else {
                    self.state.get_mut().select(Some(0));
                }
            }

            Ok(AppKey::Down) => {
                if let Some(curr) = self.state.get().selected() {
                    let len = self.list.len();
                    if curr != usize::MAX && curr != len - 1 {
                        self.state.get_mut().select(Some(curr + 1));
                    } else {
                        self.state.get_mut().select(Some(0));
                    }
                } else {
                    self.state.get_mut().select(Some(0));
                }
            }

            Ok(AppKey::Select) => {
                if let Some(selected_idx) = self.state.get().selected() {
                    if self.expanded_item_indexes.contains(&selected_idx) {
                        self.expanded_item_indexes.remove(&selected_idx);
                    } else {
                        self.expanded_item_indexes.insert(selected_idx);
                    }

                    let items: Vec<_> = self
                        .list_items_shadow
                        .iter()
                        .enumerate()
                        .map(|(i, item)| item.as_list_item(self.expanded_item_indexes.contains(&i)))
                        .collect();

                    self.list = self.list.clone().items(items);

                    return true;
                }

                return false;
            }

            _ => {
                return false;
            }
        }

        true
    }
}
