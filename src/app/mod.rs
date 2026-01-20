use anyhow::Result;
use ratatui::{DefaultTerminal, Frame, widgets::WidgetRef};
use tokio::sync::mpsc;

use crate::{
    api::ApiClient,
    config::Config,
    view::View,
};

pub mod events;
pub mod handlers;
pub mod init;
pub mod view;

// Re-export AppEvent for convenience
pub use events::AppEvent;

pub struct App {
    pub view: View,
    pub(crate) exit: bool,
    #[allow(dead_code)] // this is read, idk why it warns
    api_client: ApiClient,
    pub(crate) event_rx: mpsc::UnboundedReceiver<AppEvent>,
    pub config: Config,
}

impl App {
    pub async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.view.render_ref(frame.area(), frame.buffer_mut());
    }
}
