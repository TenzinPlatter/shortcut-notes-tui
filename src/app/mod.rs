use anyhow::Result;
use ratatui::{DefaultTerminal, Frame, widgets::FrameExt};
use tokio::sync::mpsc;

use crate::{
    api::ApiClient, cache::Cache, config::Config
};

#[cfg(any())]
use crate::view::View;

pub mod cmd;
#[cfg(any())]
pub mod events;
#[cfg(any())]
pub mod handlers;
pub mod init;
pub mod model;
pub mod msg;
pub mod pane;
pub mod update;
pub mod view;

#[cfg(any())]
pub use events::AppEvent;

pub struct App {
    pub model: model::Model,
    pub exit: bool,
    pub receiver: mpsc::UnboundedReceiver<msg::Msg>,
    pub sender: mpsc::UnboundedSender<msg::Msg>,
    pub api_client: ApiClient,
    #[cfg(any())]
    #[allow(dead_code)]
    pub view: View,
    #[allow(dead_code)]
    pub config: Config,
    #[allow(dead_code)]
    pub cache: Cache,
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
                        &self.model.config,
                        &mut self.model.cache,
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

    fn draw(&self, frame: &mut Frame) {
        use ratatui::layout::{Constraint, Direction, Layout};
        use crate::view::{epic::EpicView, story_list::StoryListView};

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(frame.area());

        let story_list_view = StoryListView::new(
            &self.model.data.stories,
            &self.model.ui.story_list,
            self.model.ui.focused_pane == model::PaneId::StoryList,
        );
        frame.render_widget_ref(story_list_view, chunks[0]);

        let epic_view = EpicView::new(
            &self.model.ui.epic_pane,
            self.model.ui.focused_pane == model::PaneId::Epic,
        );
        frame.render_widget_ref(epic_view, chunks[1]);
    }
}
