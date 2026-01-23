use anyhow::Result;
use ratatui::{DefaultTerminal, Frame, widgets::WidgetRef};
use tokio::sync::mpsc;

use crate::{
    api::ApiClient, cache::Cache, config::Config, view::View
};

pub mod events;
pub mod handlers;
pub mod init;
pub mod view;

// Re-export AppEvent for convenience
pub use events::AppEvent;

pub struct App {
    pub view: View,
    pub exit: bool,
    pub api_client: ApiClient,
    pub reciever: mpsc::UnboundedReceiver<AppEvent>,
    pub sender: mpsc::UnboundedSender<AppEvent>,
    pub config: Config,
    pub cache: Cache,
}

impl App {
    pub async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(self.sender.clone(), terminal).await?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.view.render_ref(frame.area(), frame.buffer_mut());
    }
}
