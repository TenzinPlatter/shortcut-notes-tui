use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols::border,
    text::Text,
    widgets::{Block, Paragraph, Widget, WidgetRef},
};

use crate::{
    error_display::ErrorSeverity, keys::KeyHandler, pane::Selectable,
};

pub struct ErrorPane {
    title: String,
    message: String,
    details: Option<String>,
    severity: ErrorSeverity,
    is_selected: bool,
    show_details: bool,
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
}

impl WidgetRef for ErrorPane {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let color = match self.severity {
            ErrorSeverity::Blocking => Color::Red,
            ErrorSeverity::Notification => Color::Yellow,
        };

        let text = if self.show_details {
            self.details.as_ref().unwrap_or(&self.message)
        } else {
            &self.message
        };

        let paragraph = Paragraph::new(Text::from(text.as_str()))
            .style(Style::default().fg(color));

        let mut block = Block::bordered()
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(color))
            .border_set(border::THICK);

        if self.details.is_some() {
            block = block.title_bottom(" Press 'd' for details ");
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
