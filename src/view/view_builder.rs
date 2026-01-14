use ratatui::layout::{Constraint, Direction};

use crate::view::{View, ViewSection};

#[derive(Default)]
pub struct ViewBuilder {
    sections: Vec<(Box<dyn ViewSection>, Constraint)>,
    /// the internally selected section
    selected_section: usize,
    /// whether the entire view is selected
    is_selected: bool,
    direction: Direction,
}

impl<T: ViewSection, const N: usize> From<[T; N]> for ViewBuilder {
    fn from(sections: [T; N]) -> Self {
        Self {
            sections: sections
                .into_iter()
                .map(|v| (Box::new(v) as Box<dyn ViewSection>, Constraint::Ratio(1, 1)))
                .collect(),
            selected_section: 0,
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

    pub fn add_section_with_constraint(
        mut self,
        section: impl ViewSection,
        constraint: Constraint,
    ) -> Self {
        self.sections.push((Box::new(section), constraint));
        self
    }

    pub fn add_section(mut self, new: impl ViewSection) -> Self {
        self.sections.push((Box::new(new), Constraint::Ratio(1, 1)));
        self
    }

    pub fn add_sections<I, T>(mut self, new: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: ViewSection,
    {
        for s in new.into_iter() {
            self = self.add_section(s);
        }

        self
    }

    pub fn build(mut self) -> View {
        if self.selected_section < self.sections.len() {
            self.sections[self.selected_section].0.select();
        }

        View {
            sections: self.sections,
            selected_section: self.selected_section,
            last_selected_section: 0,
            is_selected: self.is_selected,
            direction: self.direction,
        }
    }
}
