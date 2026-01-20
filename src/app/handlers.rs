use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};

use crate::{app::App, keys::{AppKey, KeyHandler}};

impl App {
    pub(super) async fn handle_events(&mut self) -> Result<()> {
        tokio::select! {
            // Poll for terminal events with a timeout to avoid blocking forever
            terminal_event = tokio::task::spawn_blocking(|| {
                if event::poll(std::time::Duration::from_millis(100))? {
                    event::read()
                } else {
                    Ok(Event::Resize(0, 0)) // Dummy event to indicate no real event
                }
            }) => {
                match terminal_event?? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        // Dismiss notification on any key
                        if key_event.code == AppKey::Quit.into() {
                            self.exit = true;
                        } else {
                            self.view.handle_key_event(key_event);
                        }
                    }
                    _ => {}
                }
            }

            // Handle app events from background tasks
            Some(app_event) = self.event_rx.recv() => {
                self.handle_app_event(app_event)?;
            }
        }

        Ok(())
    }
}
