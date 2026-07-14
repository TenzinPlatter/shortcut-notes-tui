use ratatui::layout::Constraint;

use super::PaneTrait;

pub struct Pane {
    pub view_section: Box<dyn PaneTrait>,
    pub constraint: Constraint,
    pub is_selectable: bool,
}
