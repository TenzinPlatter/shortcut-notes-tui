use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, WidgetRef},
};

use crate::{
    block::Selectable,
    keys::{AppKey, KeyHandler},
};

pub struct ParagraphBlock {
    paragraph: Paragraph<'static>,
    is_selected: bool,
}

impl Default for ParagraphBlock {
    fn default() -> Self {
        // returns static lifetime as the &str.into() calls wrap a Span<'a> around the &str's lifetime
        // which is 'static
        let counter_instructions = Line::from(vec![
            " Decrement: ".into(),
            AppKey::Up.to_string().blue().bold(),
            " Increment: ".into(),
            AppKey::Down.to_string().blue().bold(),
        ]);

        let navigation_instructions = Line::from(vec![
            " Left: ".into(),
            AppKey::Left.to_string().blue().bold(),
            " Right: ".into(),
            AppKey::Right.to_string().blue().bold(),
        ]);

        let quit_instructions = Line::from(vec![" Quit: ".into(), "<Q> ".blue().bold()]);

        let paragraph = Paragraph::new(Text::from(vec![
            counter_instructions,
            navigation_instructions,
            quit_instructions,
        ]));

        Self {
            paragraph,
            is_selected: false,
        }
    }
}

impl WidgetRef for ParagraphBlock {
    #[doc = " Draws the current state of the widget in the given buffer. That is the only method required"]
    #[doc = " to implement a custom widget."]
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from("Instructions".bold()).centered())
            .border_set(border::THICK)
            .border_style(if self.is_selected() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        self.paragraph
            .clone()
            .block(block)
            .centered()
            .render(area, buf);
    }
}

impl KeyHandler for ParagraphBlock {
    fn handle_key_event(&mut self, _key_event: crossterm::event::KeyEvent) -> bool {
        false
        // nada
    }
}

impl Selectable for ParagraphBlock {
    fn is_selected(&self) -> bool {
        self.is_selected
    }

    fn select(&mut self) {
        self.is_selected = true;
    }

    fn unselect(&mut self) {
        self.is_selected = false
    }
}
