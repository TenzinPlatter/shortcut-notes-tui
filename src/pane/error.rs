use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, WidgetRef},
};

use crate::{
    error_display::ErrorSeverity,
    keys::KeyHandler,
    pane::{ParagraphPane, Selectable},
    view::{View, ViewBuilder},
};

#[derive(Clone)]
pub struct ErrorPane {
    title: String,
    message: String,
    details: Option<String>,
    severity: ErrorSeverity,
    is_selected: bool,
    pub show_details: bool,
}

impl ErrorPane {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        severity: ErrorSeverity,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            details: None,
            severity,
            is_selected: false,
            show_details: false,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn details_pane(&self) -> Option<ParagraphPane> {
        if let Some(details) = &self.details {
            let lines = vec![Line::from(" Error Details: "), Line::from(details.clone())];
            let paragraph = Paragraph::new(Text::from(lines))
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::bordered()
                        .border_style(Style::default().fg(Color::Red))
                        .border_set(border::THICK)
                        .title(" Error details: ")
                        .title_bottom(" Press 'd' to hide details "),
                );

            Some(ParagraphPane::from(paragraph))
        } else {
            None
        }
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    pub fn as_view(&self) -> View {
        let builder = ViewBuilder::default().add_non_selectable(self.clone());

        if self.show_details
            && let Some(details_pane) = self.details_pane()
        {
            builder
                .add_non_selectable_with_constraint(
                    details_pane,
                    ratatui::layout::Constraint::Percentage(70),
                )
                .build()
        } else {
            builder.build()
        }
    }
}

impl WidgetRef for ErrorPane {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let color = match self.severity {
            ErrorSeverity::Blocking => Color::Red,
            ErrorSeverity::Notification => Color::Yellow,
        };

        let paragraph =
            Paragraph::new(Text::from(self.message.as_str())).style(Style::default().fg(color));

        let mut block = Block::bordered()
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(color))
            .border_set(border::THICK);

        if self.details.is_some() && !self.show_details {
            block = block.title_bottom(" Press 'd' to show details ");
        }

        paragraph.block(block).centered().render(area, buf);
    }
}

impl KeyHandler for ErrorPane {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if self.details.is_some() && key_event.code == KeyCode::Char('d') {
            self.show_details = !self.show_details;
            true
        } else {
            false
        }
    }
}

impl Selectable for ErrorPane {
    fn is_selected(&self) -> bool {
        self.is_selected
    }

    fn select(&mut self) {
        self.is_selected = true;
    }

    fn unselect(&mut self) {
        self.is_selected = false;
    }
}
