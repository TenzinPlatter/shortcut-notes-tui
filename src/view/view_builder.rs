use ratatui::layout::{Constraint, Direction};
use reqwest::retry::Builder;

use crate::view::{Pane, PaneTrait, View};

#[derive(Default)]
pub struct ViewBuilder {
    panes: Vec<Pane>,
    /// the internally selected section
    selected_section_idx: usize,
    /// whether the entire view is selected
    is_selected: bool,
    direction: Direction,
}

impl<T: PaneTrait, const N: usize> From<[T; N]> for ViewBuilder {
    fn from(sections: [T; N]) -> Self {
        Self {
            panes: sections
                .into_iter()
                .map(|v| Pane {
                    view_section: Box::new(v) as Box<dyn PaneTrait>,
                    constraint: Constraint::Ratio(1, 1),
                    is_selectable: true,
                })
                .collect(),
            selected_section_idx: 0,
            is_selected: false,
            direction: Direction::default(),
        }
    }
}

impl ViewBuilder {
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn select(mut self) -> Self {
        self.is_selected = true;
        self
    }

    pub fn add_selectable(mut self, pane: impl PaneTrait) -> Self {
        self.panes.push(Pane {
            view_section: Box::new(pane),
            constraint: Constraint::Ratio(1, 1),
            is_selectable: true,
        });
        self
    }

    pub fn add_selectable_with_constraint(
        mut self,
        pane: impl PaneTrait,
        constraint: Constraint,
    ) -> Self {
        self.panes.push(Pane {
            view_section: Box::new(pane),
            constraint,
            is_selectable: true,
        });
        self
    }

    pub fn add_non_selectable(mut self, pane: impl PaneTrait) -> Self {
        self.panes.push(Pane {
            view_section: Box::new(pane),
            constraint: Constraint::Ratio(1, 1),
            is_selectable: false,
        });
        self
    }

    pub fn add_non_selectable_with_constraint(
        mut self,
        section: impl PaneTrait,
        constraint: Constraint,
    ) -> Self {
        self.panes.push(Pane {
            view_section: Box::new(section),
            constraint,
            is_selectable: false,
        });
        self
    }

    pub fn add_panes<I, T>(mut self, panes: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: PaneTrait,
    {
        for section in panes.into_iter() {
            self = self.add_selectable(section);
        }
        self
    }

    pub fn build(mut self) -> View {
        if let Some(section) = self.panes.get_mut(self.selected_section_idx)
            && section.is_selectable
        {
            section.view_section.select()
        }

        View {
            panes: self.panes,
            selected_section_idx: self.selected_section_idx,
            last_selected_section: 0,
            is_selected: self.is_selected,
            direction: self.direction,
        }
    }
}

impl From<View> for ViewBuilder {
    fn from(view: View) -> Self {
        ViewBuilder {
            panes: view.panes,
            selected_section_idx: view.selected_section_idx,
            is_selected: view.is_selected,
            direction: view.direction,
        }
    }
}
