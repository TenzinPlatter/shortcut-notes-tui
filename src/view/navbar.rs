use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, WidgetRef},
};

use crate::app::model::ViewType;

pub struct NavBar {
    active_view: ViewType,
}

impl NavBar {
    pub fn new(active_view: ViewType) -> Self {
        Self { active_view }
    }
}

impl WidgetRef for NavBar {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let all_views = [
            ViewType::Iteration,
            ViewType::Epics,
            ViewType::Notes,
            ViewType::Search,
        ];

        let mut spans = Vec::new();

        for (i, view_type) in all_views.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" │ "));
            }

            let label = view_type.label();
            let span = if *view_type == self.active_view {
                Span::styled(
                    label,
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw(label)
            };

            spans.push(span);
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).block(Block::bordered());

        Widget::render(paragraph, area, buf);
    }
}
