use std::option::IntoIter;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    widgets::{FrameExt, WidgetRef},
};

use crate::{
    keys::{AppKey, KeyHandler},
    view::{View, ViewBuilder, ViewSection},
};

pub struct App {
    pub view: View,
    exit: bool,
}

impl From<View> for App {
    fn from(view: View) -> Self {
        Self { view, exit: false }
    }
}

impl<T: ViewSection> From<IntoIter<T>> for App {
    fn from(views: IntoIter<T>) -> Self {
        Self {
            view: ViewBuilder::default().add_sections(views).build(),
            exit: false,
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?; // blocks until an event occurs, thus only draw on change
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget_ref(self, frame.area());
    }

    fn handle_events(&mut self) -> anyhow::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if key_event.code == AppKey::Quit.into() {
                    self.exit = true;
                } else {
                    self.view.handle_key_event(key_event);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl WidgetRef for &App {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.view.render_ref(area, buf);
    }
}
