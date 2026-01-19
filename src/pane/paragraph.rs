use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, WidgetRef},
};

use crate::{
    api::{epic::Epic, story::Story},
    keys::{AppKey, KeyHandler},
    pane::Selectable,
};

pub struct ParagraphPane {
    paragraph: Paragraph<'static>,
    block: Block<'static>,
    is_selected: bool,
}

impl ParagraphPane {
    pub fn story(story: &Story) -> Self {
        let lines = vec![
            Line::from(format!("Name: {}", story.name)),
            Line::from(format!("Description: {}", story.description)),
        ];

        let paragraph = Paragraph::new(lines);
        let block = Block::bordered().border_set(border::THICK);

        Self {
            paragraph,
            block,
            is_selected: false,
        }
    }

    pub fn epic(epic: &Epic) -> Self {
        let lines = vec![
            Line::from(format!("ID: {}", epic.id)),
            Line::from(format!("Name: {}", epic.name)),
            Line::from(format!("Description: {}", epic.description)),
        ];

        let paragraph = Paragraph::new(lines);
        let block = Block::bordered().border_set(border::THICK);

        Self {
            paragraph,
            block,
            is_selected: false,
        }
    }

    pub fn not_authenticated() -> Self {
        let paragraph = Paragraph::new(Text::from(Line::from(
            " The SHORTCUT_API_TOKEN environment variable is not set. Please set it to your Shortcut API token to authenticate. "
                .red()
                .italic(),
        )));

        let block = Block::bordered().border_set(border::THICK);

        Self {
            paragraph,
            block,
            is_selected: false,
        }
    }

    pub fn instructions() -> Self {
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

        let block = Block::bordered()
            .title(" Instructions ".bold().underlined().into_centered_line())
            .border_set(border::THICK);

        Self {
            paragraph,
            block,
            is_selected: false,
        }
    }

    pub fn loading() -> Self {
        let paragraph = Paragraph::new("Loading stories...");
        let block = Block::bordered().border_set(border::THICK);

        Self {
            paragraph,
            block,
            is_selected: false,
        }
    }
}

impl From<Paragraph<'static>> for ParagraphPane {
    fn from(paragraph: Paragraph<'static>) -> Self {
        let block = Block::bordered().border_set(border::THICK);
        Self {
            paragraph,
            block,
            is_selected: false,
        }
    }
}

impl WidgetRef for ParagraphPane {
    #[doc = " Draws the current state of the widget in the given buffer. That is the only method required"]
    #[doc = " to implement a custom widget."]
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.paragraph
            .clone()
            .block(self.block.clone())
            .centered()
            .render(area, buf);
    }
}

impl KeyHandler for ParagraphPane {}

impl Selectable for ParagraphPane {
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
