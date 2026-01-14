use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::WidgetRef,
};

use crate::{
    block::Selectable,
    keys::{AppKey, KeyHandler},
};

mod view_builder;
pub use view_builder::ViewBuilder;

// Type alias to avoid repeating trait bounds
pub trait ViewSection: WidgetRef + KeyHandler + Selectable + 'static {}
// Blanket implementation for any type that satisfies the bounds
impl<T: WidgetRef + KeyHandler + Selectable + 'static> ViewSection for T {}

#[derive(Default)]
pub struct View {
    sections: Vec<(Box<dyn ViewSection>, Constraint)>,
    // the internally selected section
    selected_section: usize,
    // this isn't optional as unselecting something that wasn't selected doesn't really have an
    // effect, so its not really worth the extra hassle
    last_selected_section: usize,
    // whether the entire view is selected
    is_selected: bool,
    direction: Direction,
}

impl View {
    fn select_section(&mut self, new: usize) {
        self.sections[self.selected_section].0.unselect();
        self.sections[new].0.select();

        self.last_selected_section = self.selected_section;
        self.selected_section = new;
    }
}

impl Selectable for View {
    fn is_selected(&self) -> bool {
        self.is_selected
    }

    fn select(&mut self) {
        self.sections[self.selected_section].0.select();
        self.is_selected = true;
    }

    fn unselect(&mut self) {
        self.sections[self.selected_section].0.unselect();
        self.is_selected = false;
    }
}

impl KeyHandler for View {
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> bool {
        let consume_navigation = self.sections.len() > 1;

        // if we have more than one section we want to consume any navigation keys
        match key_event.code.try_into() {
            Ok(AppKey::Left) if consume_navigation => {
                let new = if self.selected_section > 0 {
                    self.selected_section - 1
                } else {
                    // if we are at 0 we need to wrap around to the end
                    self.sections.len() - 1
                };

                self.select_section(new);
            }

            Ok(AppKey::Right) if consume_navigation => {
                let new = if self.selected_section < u8::MAX.into() {
                    (self.selected_section + 1) % self.sections.len()
                } else {
                    // we are at u8::MAX, so we wrap to the start
                    0
                };

                self.select_section(new);
            }

            _ => {
                for (section, _) in self.sections.iter_mut() {
                    if section.handle_key_event(key_event) {
                        return true;
                    }
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
        let sections_len = self.sections.len();
        // TODO: maybe handle this in future?
        assert!(sections_len != 0);
        assert!(self.selected_section < sections_len);

        let layout = Layout::default()
            .direction(self.direction)
            .constraints(
                self.sections
                    .iter()
                    .map(|(_, constraint)| *constraint)
                    .collect::<Vec<_>>(),
            )
            .split(area);

        for ((section, _), area) in self.sections.iter().zip(layout.iter()) {
            section.render_ref(*area, buf);
        }
    }
}
