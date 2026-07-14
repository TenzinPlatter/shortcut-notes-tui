use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Paragraph, Widget, WidgetRef},
};

use crate::app::model::AddTodoModalState;
use crate::view::description_modal::centered_rect;

pub struct AddTodoModal<'a> {
    state: &'a AddTodoModalState,
}

impl<'a> AddTodoModal<'a> {
    pub fn new(state: &'a AddTodoModalState) -> Self {
        Self { state }
    }
}

impl WidgetRef for AddTodoModal<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let popup_area = centered_rect(50, 30, area);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" New Todo ");

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // spacer
                Constraint::Length(1), // label
                Constraint::Length(1), // input
                Constraint::Length(1), // spacer
                Constraint::Length(1), // hint
            ])
            .split(inner);

        let label = Line::from("Todo:").style(Style::default().add_modifier(Modifier::BOLD));
        buf.set_line(chunks[1].x, chunks[1].y, &label, chunks[1].width);

        let input_text = format!("{}_", self.state.input);
        let input_line = Line::from(input_text);
        buf.set_line(chunks[2].x, chunks[2].y, &input_line, chunks[2].width);

        let hint =
            Paragraph::new("Enter to add  Esc to cancel").style(Style::default().dark_gray());
        hint.render(chunks[4], buf);
    }
}
