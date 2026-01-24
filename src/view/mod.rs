use ratatui::{
    buffer::Buffer,
    layout::{Direction, Layout, Rect},
    widgets::WidgetRef,
};

use crate::{
    pane::Selectable,
    keys::{AppKey, KeyHandler},
};

pub mod story_list;

mod section;
mod view_builder;

pub use section::Pane;
pub use view_builder::ViewBuilder;

pub trait PaneTrait: WidgetRef + KeyHandler + Selectable + 'static {}
impl<T: WidgetRef + KeyHandler + Selectable + 'static> PaneTrait for T {}

#[derive(Default)]
pub struct View {
    panes: Vec<Pane>,
    // the internally selected section
    selected_section_idx: usize,
    // this isn't optional as unselecting something that wasn't selected doesn't really have an
    // effect, so its not really worth the extra hassle
    last_selected_section: usize,
    // whether the entire view is selected
    is_selected: bool,
    direction: Direction,
}

impl View {

    
    fn select_section(&mut self, new: usize) {
        self.panes[self.selected_section_idx].view_section.unselect();
        self.panes[new].view_section.select();

        self.last_selected_section = self.selected_section_idx;
        self.selected_section_idx = new;
    }

    fn next_selectable_section(&self, current: usize) -> Option<usize> {
        for i in (current + 1)..self.panes.len() {
            if self.panes[i].is_selectable {
                return Some(i);
            }
        }

        // Wrap around to the beginning
        (0..current).find(|&i| self.panes[i].is_selectable)
    }

    fn prev_selectable_section(&self, current: usize) -> Option<usize> {
        for i in (0..current).rev() {
            if self.panes[i].is_selectable {
                return Some(i);
            }
        }
        // Wrap around to the end
        (current + 1..self.panes.len())
            .rev()
            .find(|&i| self.panes[i].is_selectable)
    }
}

impl Selectable for View {
    fn is_selected(&self) -> bool {
        self.is_selected
    }

    fn select(&mut self) {
        self.panes[self.selected_section_idx].view_section.select();
        self.is_selected = true;
    }

    fn unselect(&mut self) {
        self.panes[self.selected_section_idx].view_section.unselect();
        self.is_selected = false;
    }

    fn num_selectable_children(&self) -> usize {
        self.panes
            .iter()
            .filter(|section| section.is_selectable)
            .count()
    }
}

impl KeyHandler for View {
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> bool {
        let consume_navigation = self.num_selectable_children() > 1;

        // if we have more than one section we want to consume any navigation keys
        match key_event.code.try_into() {
            Ok(AppKey::Left) if consume_navigation => {
                if let Some(new) = self.prev_selectable_section(self.selected_section_idx) {
                    self.select_section(new);
                }
            }

            Ok(AppKey::Right) if consume_navigation => {
                if let Some(new) = self.next_selectable_section(self.selected_section_idx) {
                    self.select_section(new);
                }
            }

            _ => {
                if let Some(section) = self.panes.get_mut(self.selected_section_idx) {
                    return section.view_section.handle_key_event(key_event);
                }
            }
        };

        consume_navigation
    }
}

impl WidgetRef for View {
    #[doc = " Draws the current state of the widget in the given buffer. That is the only method required"]
    #[doc = " to implement a custom widget."]
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let sections_len = self.panes.len();
        // TODO: maybe handle this in future?
        assert!(sections_len != 0);
        assert!(self.selected_section_idx < sections_len);

        let layout = Layout::default()
            .direction(self.direction)
            .constraints(
                self.panes
                    .iter()
                    .map(|section| section.constraint)
                    .collect::<Vec<_>>(),
            )
            .split(area);

        for (section, area) in self.panes.iter().zip(layout.iter()) {
            section.view_section.render_ref(*area, buf);
        }
    }
}
