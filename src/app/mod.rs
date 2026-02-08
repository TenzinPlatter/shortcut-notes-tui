use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Clear, Paragraph, StatefulWidget, WidgetRef};
use ratatui::{DefaultTerminal, Frame, widgets::FrameExt};
use tokio::sync::mpsc;

use crate::app::pane::action_menu::ActionMenu;
use crate::error::ERROR_NOTIFICATION_MAX_HEIGHT;
use crate::view::{navbar::NavBar, story_list::StoryListView};
use crate::{api::ApiClient, app::model::ViewType, config::Config};

pub mod cmd;
pub mod init;
pub mod model;
pub mod msg;
pub mod pane;
pub mod update;

pub struct App {
    pub model: model::Model,
    pub exit: bool,
    pub receiver: mpsc::UnboundedReceiver<msg::Msg>,
    pub sender: mpsc::UnboundedSender<msg::Msg>,
    pub api_client: ApiClient,
    pub config: Config,
}

impl App {
    pub async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if let Some(msg) = self.poll_for_message().await? {
                let commands = self.update(msg);

                for cmd in commands {
                    cmd::execute(
                        cmd,
                        self.sender.clone(),
                        &mut self.model,
                        &self.api_client,
                        terminal,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }

    async fn poll_for_message(&mut self) -> Result<Option<msg::Msg>> {
        use crossterm::event::{self, Event, KeyEventKind};
        use std::time::Duration;

        tokio::select! {
            terminal_event = tokio::task::spawn_blocking(|| {
                if event::poll(Duration::from_millis(100))? {
                    event::read()
                } else {
                    Ok(Event::Resize(0, 0))
                }
            }) => {
                match terminal_event?? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        Ok(Some(msg::Msg::KeyPressed(key)))
                    }
                    _ => Ok(None)
                }
            }

            Some(msg) = self.receiver.recv() => {
                Ok(Some(msg))
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        // Advance spinner animation
        self.model.ui.throbber_state.calc_next();
        let tick = self.model.ui.throbber_state.index().unsigned_abs() as usize;

        // Split screen: navbar at top, main view below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Navbar: border + content + border
                Constraint::Min(0),    // Main view: everything else
            ])
            .split(frame.area());

        // Render navbar
        let has_stories = !self.model.data.stories.is_empty();
        let navbar = NavBar::new(
            self.model.ui.active_view,
            self.model.ui.loading,
            has_stories,
            tick,
        );
        frame.render_widget_ref(navbar, chunks[0]);

        // Render main view based on active_view
        match self.model.ui.active_view {
            ViewType::Iteration => {
                let story_list_view = StoryListView::new(
                    &self.model.data.stories,
                    &self.model.ui.story_list,
                    self.model.data.active_story.as_ref(),
                    true, // Always focused (single view)
                    self.model.ui.loading,
                    tick,
                );

                frame.render_widget_ref(story_list_view, chunks[1]);
            }

            ViewType::Epics | ViewType::Notes | ViewType::Search => {
                // Placeholder for future views
                let placeholder = Paragraph::new("Coming soon...").block(Block::bordered());
                frame.render_widget(placeholder, chunks[1]);
            }
        }

        if self.model.ui.action_menu.is_showing {
            let (width, height) = ActionMenu::window_dimensions();
            let (width, height) = (width as u16, height as u16);
            let x = (frame.area().width - width) / 2;
            let y = (frame.area().height - height) / 2;

            let rect = Rect::new(x, y, width, height);
            ActionMenu.render(
                rect,
                frame.buffer_mut(),
                &mut self.model.ui.action_menu.list_state,
            );
        }

        self.draw_error(frame);
    }

    fn draw_error(&self, frame: &mut Frame) {
        let mut used_height = 0;
        for error in self.model.ui.errors.iter().filter(|e| !e.is_expired()) {
            let width = error.get_required_width();
            let height = error.get_required_height(width);
            let area = frame.area();
            let area = Rect::new(
                area.width - width,
                used_height,
                width,
                height.min(ERROR_NOTIFICATION_MAX_HEIGHT),
            );

            used_height += height;

            // clear terminal area, stops characters behind empty space from being visible
            frame.render_widget(Clear, area);

            error.render_ref(area, frame.buffer_mut());
        }
    }
}
